use crate::build::MotokoImport;
use crate::github::{download_github_package, fetch_file, parse_github_url, RepoInfo};
use crate::{mops, storage, utils::create_bar};
use anyhow::{anyhow, Error, Result};
use candid::Principal;
use console::style;
use futures::future::try_join_all;
use ic_agent::Agent;
use indicatif::ProgressBar;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use toml_edit::{value, DocumentMut, ImDocument};

#[derive(Debug, Serialize, Deserialize)]
struct Package {
    name: String,
    version: Option<String>,
    source: String,
    base_dir: String,
    repo: Option<RepoInfo>,
    dependencies: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Canister {
    canister_id: String,
    name: Option<String>,
    timestamp: Option<String>,
    candid: String,
}
#[derive(Default, Serialize, Deserialize)]
struct Packages {
    package: Vec<Package>,
    canister: Option<Vec<Canister>>,
}

pub async fn update_mops_toml(agent: &Agent, libs: BTreeSet<MotokoImport>) -> Result<()> {
    let mops = Path::new("mops.toml");
    let mut doc = if mops.exists() {
        let str = fs::read_to_string(mops)?;
        str.parse::<DocumentMut>()?
    } else {
        DocumentMut::new()
    };
    let service = mops::Service(mops::CANISTER_ID, agent);
    if doc.get("dependencies").is_none() {
        doc["dependencies"] = toml_edit::table();
    }
    /*if doc.get("canister").is_none() {
        doc["canister"] = toml_edit::array();
    }*/
    let mut unknown_libs = Vec::new();
    for lib in libs {
        match lib {
            MotokoImport::Lib(lib) => {
                if doc["dependencies"].get(&lib).is_some() {
                    continue;
                }
                let version = service.get_highest_version(&lib).await?.into_result();
                match version {
                    Ok(version) => doc["dependencies"][lib] = value(version),
                    Err(_) => unknown_libs.push(lib),
                }
            }
            /*MotokoImport::Canister(name) => {
                if doc["canister"].get(&name).is_some() {
                    continue;
                }
                return Err(anyhow!("Add the canister id of \"{name}\" to the [canisters] section in mops.toml."));
            }
            MotokoImport::Ic(id) => {
                let item = id.to_string();
                if doc["canister"].get(&item).is_some() {
                    continue;
                }
                doc["canisters"][item] = value("");
            }*/
            _ => (),
        }
    }
    fs::write(mops, doc.to_string())?;
    if !unknown_libs.is_empty() {
        return Err(anyhow!("The following imports cannot be found on mops. Please manually add it to mops.toml:\n{unknown_libs:?}"));
    }
    update_mops_lock(agent).await?;
    Ok(())
}
async fn update_mops_lock(agent: &Agent) -> Result<()> {
    let lock = Path::new("mops.lock");
    let pkgs = parse_mops_lock(lock).unwrap_or_default();
    let mut map: BTreeMap<_, _> = pkgs.package.into_iter().map(|p| (p.get_key(), p)).collect();
    let mut canisters: BTreeMap<_, _> = pkgs
        .canister
        .unwrap_or_default()
        .into_iter()
        .map(|c| (c.get_key(), c))
        .collect();
    let str = fs::read_to_string(Path::new("mops.toml"))?;
    let toml = parse_mops_toml(&str)?;
    let service = mops::Service(mops::CANISTER_ID, agent);
    let bar = create_bar(toml.dependencies.len() + toml.canisters.len());
    bar.set_prefix("Updating mops.lock");
    for canister in toml.canisters {
        if canisters.contains_key(&canister.get_key()) {
            bar.inc(1);
            continue;
        }
        let (timestamp, candid) = if let Some(candid) = canister.candid {
            (None, candid)
        } else {
            use std::time::SystemTime;
            let id = Principal::from_text(&canister.canister_id)?;
            let candid = String::from_utf8(
                agent
                    .read_state_canister_metadata(id, "candid:service")
                    .await?,
            )?;
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            (Some(format!("{:?}", timestamp)), candid)
        };
        let info = Canister {
            canister_id: canister.canister_id,
            name: canister.name,
            timestamp,
            candid,
        };
        assert!(canisters.insert(info.get_key(), info).is_none());
        bar.inc(1);
    }

    let mut queue = toml.dependencies.into_iter().collect::<VecDeque<_>>();
    // TODO: maintain a map between mops to resolved package.get_key, so we can rewrite dependencies entry at the end
    while let Some(m) = queue.pop_front() {
        let pkg = match m {
            Mops::Mops { name, version } => {
                bar.set_message(name.clone());
                if map.contains_key(&format!("{name}-{version}")) {
                    bar.inc(1);
                    continue;
                }
                let pkg = service
                    .get_package_details(&name, &version)
                    .await?
                    .into_result()
                    .map_err(Error::msg)?;
                let source = pkg.publication.storage.to_string();
                let base_dir = pkg.config.base_dir;
                let dependencies = pkg
                    .config
                    .dependencies
                    .into_iter()
                    .map(|d| {
                        let name = d.name;
                        let mops = if d.version.is_empty() {
                            Mops::Repo { name, repo: d.repo }
                        } else {
                            Mops::Mops {
                                name,
                                version: d.version,
                            }
                        };
                        bar.inc_length(1);
                        let key = mops.get_display_key();
                        queue.push_back(mops);
                        key
                    })
                    .collect();
                Package {
                    name,
                    version: Some(version),
                    source,
                    base_dir,
                    repo: None,
                    dependencies,
                }
            }
            Mops::Repo { name, repo } => {
                bar.set_message(name.clone());
                let repo_info = parse_github_url(&repo).await?;
                if map.contains_key(&format!("{}-{}", name, repo_info.commit)) {
                    bar.inc(1);
                    continue;
                }
                let mut version = None;
                let dependencies = if let Ok(str) = fetch_file(&repo_info, "mops.toml").await {
                    let mops = parse_mops_toml(&str)?;
                    version = mops.version;
                    // TODO remove Mops::Local
                    mops.dependencies
                        .into_iter()
                        .map(|m| {
                            let key = m.get_display_key();
                            bar.inc_length(1);
                            queue.push_back(m);
                            key
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                if version.is_none() {
                    version = repo_info.guess_version();
                }
                Package {
                    name,
                    version,
                    source: "github".to_string(),
                    base_dir: repo_info.base_dir.clone(),
                    repo: Some(repo_info),
                    dependencies,
                }
            }
            Mops::Local { name, path } => {
                bar.set_message(name.clone());
                let toml = Path::new(&path).join("mops.toml");
                let canonicalized = fs::canonicalize(path)?;
                if map.contains_key(&format!("{name}-{}", canonicalized.display())) {
                    bar.inc(1);
                    continue;
                }
                let source = format!("file://{}", canonicalized.display());
                let mut version = None;
                let mops = if toml.exists() {
                    let str = fs::read_to_string(toml)?;
                    let mops = parse_mops_toml(&str)?;
                    version = mops.version;
                    mops.dependencies
                } else {
                    Vec::new()
                };
                Package {
                    name,
                    version,
                    source,
                    base_dir: "src".to_string(),
                    repo: None,
                    dependencies: mops
                        .into_iter()
                        .map(|m| {
                            let key = m.get_display_key();
                            bar.inc_length(1);
                            queue.push_back(m);
                            key
                        })
                        .collect(),
                }
            }
        };
        assert!(map.insert(pkg.get_key(), pkg).is_none());
        bar.inc(1);
    }
    bar.finish_and_clear();
    let pkgs = resolve_versions(map)?;
    let mut res = DocumentMut::new();
    let mut pkg_array = toml_edit::ArrayOfTables::new();
    for p in pkgs {
        let d = toml_edit::ser::to_document(&p)?;
        pkg_array.push(d.as_table().clone());
    }
    res.insert("package", toml_edit::Item::ArrayOfTables(pkg_array));
    let mut can_array = toml_edit::ArrayOfTables::new();
    for c in canisters.into_values() {
        let d = toml_edit::ser::to_document(&c)?;
        can_array.push(d.as_table().clone());
    }
    res.insert("canister", toml_edit::Item::ArrayOfTables(can_array));
    use std::io::Write;
    let mut buf = fs::File::create(lock)?;
    buf.write_all(
        b"# This file is auto-generated by mops.\n# It is not intended for manual editing.\n\n",
    )?;
    buf.write_all(res.to_string().as_bytes())?;
    Ok(())
}
fn resolve_versions(map: BTreeMap<String, Package>) -> Result<Vec<Package>> {
    let mut res: BTreeMap<String, Package> = BTreeMap::new();
    for pkg in map.into_values() {
        if let Some(e) = res.get(&pkg.name) {
            match (&e.version, &pkg.version) {
                (None, _) | (_, None) => return Err(anyhow!(resolve_error(e, &pkg))),
                (Some(ve), Some(vp)) => match (parse_version(ve), parse_version(vp)) {
                    (None, _) | (_, None) => return Err(anyhow!(resolve_error(e, &pkg))),
                    (Some(ve), Some(vp)) => {
                        if ve < vp {
                            res.insert(pkg.name.clone(), pkg);
                        }
                    }
                },
            }
        } else {
            res.insert(pkg.name.clone(), pkg);
        }
    }
    Ok(res.into_values().collect())
}
fn parse_version(ver: &str) -> Option<Version> {
    ver.parse::<Version>().ok()
}
fn resolve_error(p1: &Package, p2: &Package) -> String {
    let p1 = toml_edit::ser::to_string(p1).unwrap();
    let p2 = toml_edit::ser::to_string(p2).unwrap();
    format!(
        "Version conflict:\n{}\nand\n\n{}",
        style(&p1).green(),
        style(&p2).green()
    )
}
pub fn generate_moc_args(base_path: &Path) -> Result<Vec<String>> {
    let lock = parse_mops_lock(Path::new("mops.lock")).unwrap_or_default();
    let mut args: Vec<_> = lock
        .package
        .into_iter()
        .flat_map(|pkg| {
            let path = base_path
                .join(pkg.get_path())
                .join(pkg.base_dir)
                .to_string_lossy()
                .to_string();
            vec!["--package".to_string(), pkg.name, path]
        })
        .collect();
    if let Some(canisters) = lock.canister {
        if !canisters.is_empty() {
            args.extend_from_slice(&["--actor-idl".to_string(), ".mops/candid".to_string()]);
        }
        for c in canisters {
            let file = Path::new(".mops/candid").join(format!("{}.did", c.canister_id));
            fs::create_dir_all(file.parent().unwrap())?;
            if c.timestamp.is_none() {
                let candid = fs::read_to_string(c.candid)?;
                fs::write(file, candid)?;
            } else {
                fs::write(file, c.candid)?;
            }
            if let Some(name) = c.name {
                args.extend_from_slice(&["--actor-alias".to_string(), name, c.canister_id]);
            }
        }
    }
    Ok(args)
}
pub async fn download_packages_from_lock(agent: &Agent, root: &Path) -> Result<()> {
    let lock = Path::new("mops.lock");
    let pkgs = parse_mops_lock(lock)?.package;
    let service = Rc::new(mops::Service(mops::CANISTER_ID, agent));
    let bar = Rc::new(create_bar(pkgs.len()));
    bar.set_prefix("Downloading packages");
    let mut mop_futures = Vec::new();
    let mut git_futures = Vec::new();
    for pkg in pkgs {
        bar.set_message(pkg.name.clone());
        let subpath = pkg.get_path();
        let path = root.join(subpath);
        if path.join(pkg.get_done_file()).exists() {
            bar.inc(1);
            continue;
        }
        match pkg.get_type() {
            PackageType::Mops { id, .. } => {
                let id = Principal::from_text(id)?;
                mop_futures.push(download_mops_package(
                    path,
                    pkg.name,
                    pkg.version.unwrap(),
                    service.clone(),
                    id,
                    bar.clone(),
                ));
            }
            PackageType::Repo(_) => {
                git_futures.push(download_github_package(
                    path,
                    pkg.repo.unwrap(),
                    bar.clone(),
                ));
            }
            PackageType::Local(_) => {
                bar.inc(1);
            }
        }
    }
    try_join_all(mop_futures).await?;
    try_join_all(git_futures).await?;
    bar.finish_and_clear();
    Ok(())
}
async fn download_mops_package(
    base_path: PathBuf,
    lib: String,
    version: String,
    service: Rc<mops::Service<'_>>,
    storage_id: Principal,
    bar: Rc<ProgressBar>,
) -> Result<()> {
    let ids = service
        .get_file_ids(&lib, &version)
        .await?
        .into_result()
        .map_err(Error::msg)?;
    let mut futures = Vec::new();
    let storage = Rc::new(storage::Service(storage_id, service.1));
    for id in ids {
        futures.push(download_file(base_path.clone(), id, storage.clone()));
    }
    try_join_all(futures).await?;
    fs::write(base_path.join("DONE"), "")?;
    bar.println(format!(
        "{:>12} {lib}@{version}",
        style("Downloaded").green().bold()
    ));
    bar.inc(1);
    Ok(())
}
async fn download_file(
    base_path: PathBuf,
    id: String,
    storage: Rc<storage::Service<'_>>,
) -> Result<()> {
    let meta = storage
        .get_file_meta(&id)
        .await?
        .into_result()
        .map_err(Error::msg)?;
    let mut blob = Vec::new();
    for i in 0..meta.chunk_count {
        let chunk = storage
            .download_chunk(&id, &i.into())
            .await?
            .into_result()
            .map_err(Error::msg)?;
        blob.extend(chunk);
    }
    let path = base_path.join(meta.path);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, blob)?;
    Ok(())
}
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize)]
enum Mops {
    Mops { name: String, version: String },
    Repo { name: String, repo: String },
    Local { name: String, path: String },
}
#[derive(Debug, Serialize, Deserialize)]
struct CanisterInfo {
    canister_id: String,
    name: Option<String>,
    candid: Option<String>,
}
#[derive(Debug)]
struct MopsConfig {
    version: Option<String>,
    dependencies: Vec<Mops>,
    canisters: Vec<CanisterInfo>,
}
fn parse_mops_toml(str: &str) -> Result<MopsConfig> {
    let doc = str.parse::<ImDocument<_>>()?;
    let mut mops = Vec::new();
    let mut version = None;
    if let Some(pkg) = doc.get("package") {
        if let Some(ver) = pkg.get("version") {
            version = Some(ver.as_value().unwrap().as_str().unwrap().to_string());
        }
    }
    if let Some(deps) = doc.get("dependencies") {
        let deps = deps
            .as_table()
            .ok_or_else(|| anyhow!("invalid dependencies"))?;
        for (lib, version) in deps.iter() {
            let version = version
                .as_value()
                .ok_or_else(|| anyhow!("invalid version"))?
                .as_str()
                .ok_or_else(|| anyhow!("invalid version"))?;
            if version.starts_with("https://github.com") {
                mops.push(Mops::Repo {
                    name: lib.to_string(),
                    repo: version.to_string(),
                });
            } else if Path::new(version).exists() {
                mops.push(Mops::Local {
                    name: lib.to_string(),
                    path: version.to_string(),
                });
            } else {
                mops.push(Mops::Mops {
                    name: lib.to_string(),
                    version: version.to_string(),
                });
            }
        }
    }
    let mut canisters = Vec::new();
    if let Some(item) = doc.get("canister") {
        for canister in item.as_array_of_tables().unwrap().iter() {
            let canister_id = canister
                .get("canister_id")
                .ok_or_else(|| anyhow!("canister_id is required"))?
                .as_value()
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let name = canister
                .get("name")
                .map(|name| name.as_value().unwrap().as_str().unwrap().to_string());
            let candid = canister
                .get("candid")
                .map(|name| name.as_value().unwrap().as_str().unwrap().to_string());
            canisters.push(CanisterInfo {
                canister_id,
                name,
                candid,
            });
        }
    }
    Ok(MopsConfig {
        version,
        dependencies: mops,
        canisters,
    })
}
fn parse_mops_lock(lock: &Path) -> Result<Packages> {
    let str = fs::read_to_string(lock)?;
    let doc = str.parse::<ImDocument<_>>()?;
    let lock = toml_edit::de::from_document::<Packages>(doc)?;
    Ok(lock)
}
enum PackageType<'a> {
    Mops { ver: &'a str, id: &'a str },
    Local(&'a str),
    Repo(&'a RepoInfo),
}
impl Package {
    fn get_type(&self) -> PackageType {
        if self.source.starts_with("file://") {
            let local = self.source.strip_prefix("file://").unwrap();
            PackageType::Local(local)
        } else if self.source == "github" {
            PackageType::Repo(self.repo.as_ref().unwrap())
        } else {
            PackageType::Mops {
                ver: self.version.as_ref().unwrap(),
                id: &self.source,
            }
        }
    }
    fn get_key(&self) -> String {
        // Make sure this is the same logic as used in update_mops_lock
        match self.get_type() {
            PackageType::Mops { ver, .. } => format!("{}-{}", self.name, ver),
            PackageType::Repo(repo) => format!("{}-{}", self.name, repo.commit),
            PackageType::Local(local) => format!("{}-{}", self.name, local),
        }
    }
    fn get_path(&self) -> String {
        match self.get_type() {
            PackageType::Mops { ver, .. } => format!("mops/{}-{}", self.name, ver),
            PackageType::Repo(repo) => {
                let repo_name = repo.repo.replace('/', "-");
                format!("git/{}/{}", repo_name, &repo.commit[..8])
            }
            PackageType::Local(local) => local.to_string(),
        }
    }
    fn get_done_file(&self) -> String {
        // Make sure this returns the same name as each download function
        match self.get_type() {
            PackageType::Mops { .. } => "DONE".to_string(),
            PackageType::Repo(repo) => repo.get_done_file(),
            PackageType::Local(_) => "".to_string(),
        }
    }
}
impl Canister {
    fn get_key(&self) -> String {
        // technically it's self.name.unwrap_or(canister_id). Need to think about the logic for dedup
        format!("{}-{:?}", self.canister_id, self.name)
    }
}
impl CanisterInfo {
    fn get_key(&self) -> String {
        format!("{}-{:?}", self.canister_id, self.name)
    }
}
impl Mops {
    fn get_display_key(&self) -> String {
        // only for displaying in dependencies, not used for dedup
        match self {
            Mops::Mops { name, version } => format!("{name}-{version}"),
            Mops::Repo { name, repo } => format!("{name}-{repo}"),
            Mops::Local { name, path } => format!("{name}-{path}"),
        }
    }
}
