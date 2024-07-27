use crate::github::{download_github_package, fetch_file, parse_github_url, RepoInfo};
use crate::{mops, storage, utils::create_bar};
use anyhow::{Error, Result};
use candid::Principal;
use futures::future::try_join_all;
use ic_agent::Agent;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
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
#[derive(Default, Serialize, Deserialize)]
struct Packages {
    package: Vec<Package>,
}

pub async fn update_mops_toml(agent: &Agent, libs: Vec<&String>) -> Result<()> {
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
    for lib in libs {
        if doc["dependencies"].get(lib).is_some() {
            continue;
        }
        let version = service.get_highest_version(lib).await?.into_result();
        match version {
            Ok(version) => doc["dependencies"][lib] = value(version),
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "library {lib} not found on mops. Please manually add it to mops.toml"
                ))
            }
        }
    }
    fs::write(mops, doc.to_string())?;
    update_mops_lock(agent).await?;
    Ok(())
}
async fn update_mops_lock(agent: &Agent) -> Result<()> {
    let lock = Path::new("mops.lock");
    let doc = if lock.exists() {
        let str = fs::read_to_string(lock)?;
        let doc = str.parse::<ImDocument<_>>()?;
        toml_edit::de::from_document::<Packages>(doc)?
    } else {
        Packages::default()
    };
    let mut map: BTreeMap<_, _> = doc.package.into_iter().map(|p| (p.get_key(), p)).collect();
    let str = fs::read_to_string(Path::new("mops.toml"))?;
    let mops = parse_mops_toml(&str)?;
    let service = mops::Service(mops::CANISTER_ID, agent);
    let bar = create_bar(mops.len());
    bar.set_prefix("Updating mops.lock");
    let mut queue = mops.into_iter().collect::<VecDeque<_>>();
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
                let dependencies = if let Ok(str) = fetch_file(&repo_info, "mops.toml").await {
                    let mops = parse_mops_toml(&str)?;
                    // TODO remove Mops::Local
                    mops.into_iter()
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
                Package {
                    name,
                    version: None,
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
                let mops = if toml.exists() {
                    let str = fs::read_to_string(toml)?;
                    parse_mops_toml(&str)?
                } else {
                    Vec::new()
                };
                Package {
                    name,
                    version: None,
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
    let mut res = DocumentMut::new();
    let mut array = toml_edit::ArrayOfTables::new();
    for p in map.values() {
        let d = toml_edit::ser::to_document(p)?;
        array.push(d.as_table().clone());
    }
    res.insert("package", toml_edit::Item::ArrayOfTables(array));
    use std::io::Write;
    let mut buf = fs::File::create(lock)?;
    buf.write_all(
        b"# This file is auto-generated by mops.\n# It is not intended for manual editing.\n\n",
    )?;
    buf.write_all(res.to_string().as_bytes())?;
    Ok(())
}
pub async fn download_packages_from_lock(agent: &Agent, root: &Path) -> Result<()> {
    let lock = Path::new("mops.lock");
    let str = fs::read_to_string(lock)?;
    let doc = str.parse::<ImDocument<_>>()?;
    let lock = toml_edit::de::from_document::<Packages>(doc)?;
    let service = Rc::new(mops::Service(mops::CANISTER_ID, agent));
    let bar = Rc::new(create_bar(lock.package.len()));
    bar.set_prefix("Downloading packages");
    let mut mop_futures = Vec::new();
    let mut git_futures = Vec::new();
    for pkg in lock.package {
        bar.set_message(pkg.name.clone());
        let subpath = pkg.get_path();
        let path = root.join(subpath);
        if path.join("DONE").exists() {
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
    bar.println(format!("Downloaded {lib}@{version}"));
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
fn parse_mops_toml(str: &str) -> Result<Vec<Mops>> {
    let doc = str.parse::<ImDocument<_>>()?;
    let mut mops = Vec::new();
    if let Some(deps) = doc.get("dependencies") {
        let deps = deps
            .as_table()
            .ok_or_else(|| anyhow::anyhow!("invalid dependencies"))?;
        for (lib, version) in deps.iter() {
            let version = version
                .as_value()
                .ok_or_else(|| anyhow::anyhow!("invalid version"))?
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("invalid version"))?;
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
    Ok(mops)
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
            PackageType::Repo(repo) => format!("git/{}-{}", self.name, &repo.commit[..8]),
            PackageType::Local(local) => local.to_string(),
        }
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
