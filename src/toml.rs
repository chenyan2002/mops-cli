use crate::build::MotokoImport;
use crate::github::{download_github_package, fetch_file, parse_github_url, RepoInfo};
use crate::{
    env::Env,
    mops, storage,
    utils::{create_bar, println},
};
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
    canister_id: Option<String>,
    name: Option<String>,
    timestamp: Option<String>,
    output: Option<String>,
    candid: String,
}
#[derive(Default, Serialize, Deserialize)]
struct Packages {
    package: Vec<Package>,
    canister: Option<Vec<Canister>>,
}

pub async fn update_mops_toml(
    agent: &Agent,
    env: &Env,
    libs: BTreeSet<MotokoImport>,
) -> Result<()> {
    let mops = &env.get_mops_toml_path();
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
    if doc.get("canister").is_none() {
        doc["canister"] = toml_edit::array();
    }
    let mut unknown_libs = Vec::new();
    for lib in libs {
        match lib {
            MotokoImport::Lib(lib) => {
                if doc["dependencies"].get(&lib).is_some() {
                    continue;
                }
                let version = service.get_highest_version(&lib).await?.into_result();
                match version {
                    Ok(version) => {
                        println(
                            None,
                            "stdout",
                            &format!(
                                "{:>12} mops.toml with {lib}@{version}",
                                style("Updated").green().bold()
                            ),
                        );
                        doc["dependencies"][lib] = value(version);
                    }
                    Err(_) => unknown_libs.push(lib),
                }
            }
            MotokoImport::Canister(name) => {
                let canisters = doc["canister"].as_array_of_tables_mut().unwrap();
                if canisters.iter().any(|c| {
                    c.get("name")
                        .is_some_and(|n| n.as_value().unwrap().as_str().unwrap() == name)
                }) {
                    continue;
                }
                return Err(anyhow!("Add the following to mops.toml.\n[[canister]]\nname = \"{name}\"\ncanister_id = \"canister_id\""));
            }
            MotokoImport::Ic(id) => {
                let canisters = doc["canister"].as_array_of_tables_mut().unwrap();
                let id = id.to_string();
                if canisters.iter().any(|c| {
                    c.get("canister_id")
                        .is_some_and(|n| n.as_value().unwrap().as_str().unwrap() == id)
                }) {
                    continue;
                }
                let mut table = toml_edit::Table::new();
                println(
                    None,
                    "stdout",
                    &format!(
                        "{:>12} mops.toml with canister {id}",
                        style("Updated").green().bold()
                    ),
                );
                table.insert("canister_id", value(id.to_string()));
                canisters.push(table);
            }
            MotokoImport::Local(_) => (),
        }
    }
    fs::write(mops, doc.to_string())?;
    if !unknown_libs.is_empty() {
        return Err(anyhow!("The following imports cannot be found on mops. Please manually add it to mops.toml:\n{unknown_libs:?}"));
    }
    update_mops_lock(agent, env).await?;
    Ok(())
}
async fn update_mops_lock(agent: &Agent, env: &Env) -> Result<()> {
    let lock = env.get_mops_lock_path();
    let pkgs = parse_mops_lock(&lock).unwrap_or_default();
    let mut map: BTreeMap<_, _> = pkgs.package.into_iter().map(|p| (p.get_key(), p)).collect();
    let mut canisters: BTreeMap<_, _> = pkgs
        .canister
        .unwrap_or_default()
        .into_iter()
        .map(|c| (c.get_key(), c))
        .collect();
    let str = fs::read_to_string(env.get_mops_toml_path())?;
    let toml = parse_mops_toml(&env.project_root, &str)?;
    let service = mops::Service(mops::CANISTER_ID, agent);
    let bar = create_bar(toml.dependencies.len() + toml.canisters.len());
    bar.set_prefix("Updating mops.lock");
    for canister in toml.canisters {
        if let Some(c) = canisters.get(&canister.get_key()) {
            if c.no_need_to_update(&canister) {
                bar.inc(1);
                continue;
            }
        }
        let (timestamp, candid) = if let Some(candid) = canister.candid {
            (None, candid)
        } else {
            use std::time::SystemTime;
            // TODO handle aaaaa-aa
            let id = Principal::from_text(canister.canister_id.clone().unwrap())?;
            let candid = String::from_utf8(
                agent
                    .read_state_canister_metadata(id, "candid:service")
                    .await?,
            )?;
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            println(
                Some(&bar),
                "stdout",
                &format!(
                    "{:>12} canister interface for {id}",
                    style("Fetched").green().bold()
                ),
            );
            (Some(format!("{:?}", timestamp)), candid)
        };
        let info = Canister {
            canister_id: canister.canister_id,
            name: canister.name,
            output: canister.output,
            timestamp,
            candid,
        };
        canisters.insert(info.get_key(), info);
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
                if map.contains_key(&format!("{}-{}-{}", name, repo_info.repo, repo_info.commit)) {
                    bar.inc(1);
                    continue;
                }
                let mut version = None;
                let dependencies = if let Ok(str) = fetch_file(&repo_info, "mops.toml").await {
                    // I hope the base_path here is irrelevant, so we can just use cwd
                    let mops = parse_mops_toml(Path::new("."), &str)?;
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
                    let mops = parse_mops_toml(&canonicalized, &str)?;
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
pub fn generate_moc_args(env: &Env) -> Result<Vec<String>> {
    let lock = parse_mops_lock(&env.get_mops_lock_path()).unwrap_or_default();
    let mut args: Vec<_> = lock
        .package
        .into_iter()
        .flat_map(|pkg| {
            let path = env
                .cache_dir
                .join(pkg.get_path())
                .join(pkg.base_dir)
                .to_string_lossy()
                .to_string();
            vec!["--package".to_string(), pkg.name, path]
        })
        .collect();
    if let Some(canisters) = lock.canister {
        let idl_path = env.get_target_idl_path();
        if !canisters.is_empty() {
            args.extend_from_slice(&["--actor-idl".to_string(), idl_path.display().to_string()]);
        }
        let (with_id, type_only): (Vec<_>, Vec<_>) =
            canisters.into_iter().partition(|c| c.canister_id.is_some());
        for c in type_only {
            use candid_parser::{bindings::motoko, utils::CandidSource};
            let candid = Path::new(&c.candid);
            let output = PathBuf::from(&c.output.unwrap());
            let (env, actor) = CandidSource::File(candid).load()?;
            let binding = motoko::compile(&env, &actor);
            fs::create_dir_all(output.parent().unwrap())?;
            fs::write(output, binding)?;
        }
        for c in with_id {
            let canister_id = c.canister_id.unwrap();
            let file = idl_path.join(format!("{}.did", canister_id));
            fs::create_dir_all(file.parent().unwrap())?;
            if c.timestamp.is_none() {
                let candid = fs::read_to_string(c.candid)?;
                fs::write(file, candid)?;
            } else {
                fs::write(file, c.candid)?;
            }
            if let Some(name) = c.name {
                args.extend_from_slice(&["--actor-alias".to_string(), name, canister_id]);
            }
        }
    }
    Ok(args)
}
pub async fn update_packages_from_lock(agent: &Agent, env: &Env) -> Result<()> {
    let lock = env.get_mops_lock_path();
    let pkgs = parse_mops_lock(&lock)?.package;
    let service = Rc::new(mops::Service(mops::CANISTER_ID, agent));
    let pkgs: Vec<_> = pkgs
        .into_iter()
        .filter_map(|p| match p.get_type() {
            PackageType::Mops { .. } => Some((p.name, p.version.unwrap())),
            _ => None,
        })
        .collect();
    let mut futures = Vec::new();
    for (name, _) in &pkgs {
        futures.push(service.get_highest_version(name));
    }
    let versions = try_join_all(futures).await?;
    for (latest, (name, ver)) in versions
        .into_iter()
        .map(|v| v.into_result().map_err(Error::msg))
        .zip(pkgs.into_iter())
    {
        let latest = latest?;
        if latest == ver {
            continue;
        }
        println!("{name}@{ver} -> {latest}");
    }
    Ok(())
}
pub async fn download_packages_from_lock(agent: &Agent, env: &Env) -> Result<()> {
    let lock = env.get_mops_lock_path();
    let pkgs = parse_mops_lock(&lock)?.package;
    let service = Rc::new(mops::Service(mops::CANISTER_ID, agent));
    let bar = Rc::new(create_bar(pkgs.len()));
    bar.set_prefix("Downloading packages");
    let mut mop_futures = Vec::new();
    let mut git_futures = Vec::new();
    for pkg in pkgs {
        bar.set_message(pkg.name.clone());
        let subpath = pkg.get_path();
        let path = env.cache_dir.join(subpath);
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
    println(
        Some(&bar),
        "stdout",
        &format!("{:>12} {lib}@{version}", style("Downloaded").green().bold()),
    );
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
    canister_id: Option<String>,
    name: Option<String>,
    candid: Option<String>,
    output: Option<String>,
}
#[derive(Debug)]
struct MopsConfig {
    version: Option<String>,
    dependencies: Vec<Mops>,
    canisters: Vec<CanisterInfo>,
}
fn parse_mops_toml(base_path: &Path, str: &str) -> Result<MopsConfig> {
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
    fn get_field(table: &toml_edit::Table, field: &str) -> Option<String> {
        table
            .get(field)
            .map(|f| f.as_value().unwrap().as_str().unwrap().to_string())
    }
    fn resolve_path(base_path: &Path, path: &str) -> String {
        let mut path = PathBuf::from(path);
        if !path.is_absolute() {
            path = base_path.join(path);
        }
        fs::canonicalize(path.clone())
            .unwrap_or_else(|_| panic!("Cannot find {}", path.display()))
            .to_string_lossy()
            .to_string()
    }
    if let Some(item) = doc.get("canister") {
        for canister in item.as_array_of_tables().unwrap().iter() {
            let canister_id = get_field(canister, "canister_id");
            let name = get_field(canister, "name");
            let candid = get_field(canister, "candid").map(|p| resolve_path(base_path, &p));
            let output = get_field(canister, "output").map(|p| resolve_path(base_path, &p));
            if canister_id.is_none() {
                if candid.is_none() {
                    return Err(anyhow!(
                        "canister_id or candid is required in \"{canister}\""
                    ));
                } else if output.is_none() {
                    return Err(anyhow!("output is required in \"{canister}\""));
                }
            } else if output.is_some() {
                return Err(anyhow!("output is not allowed in \"{canister}\""));
            }
            canisters.push(CanisterInfo {
                canister_id,
                name,
                candid,
                output,
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
            PackageType::Repo(repo) => format!("{}-{}-{}", self.name, repo.repo, repo.commit),
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
        self.name
            .as_ref()
            .unwrap_or_else(|| {
                self.canister_id
                    .as_ref()
                    .unwrap_or_else(|| self.output.as_ref().unwrap())
            })
            .clone()
    }
    fn no_need_to_update(&self, new: &CanisterInfo) -> bool {
        if self.canister_id != new.canister_id {
            return false;
        }
        if self.timestamp.is_some() {
            new.candid.is_none()
        } else {
            new.candid.as_ref().is_some_and(|c| c == &self.candid)
        }
    }
}
impl CanisterInfo {
    fn get_key(&self) -> String {
        self.name
            .as_ref()
            .unwrap_or_else(|| {
                self.canister_id
                    .as_ref()
                    .unwrap_or_else(|| self.output.as_ref().unwrap())
            })
            .clone()
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
