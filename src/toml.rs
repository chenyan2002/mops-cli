use crate::mops;
use anyhow::{Error, Result};
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use toml_edit::{value, DocumentMut, ImDocument};

#[derive(Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
struct Package {
    name: String,
    version: Option<String>,
    source: String,
    dependencies: Vec<String>,
}
#[derive(Default, Serialize, Deserialize)]
struct Packages {
    package: Vec<Package>,
}

pub async fn update_mops_toml(agent: &Agent, libs: Vec<&String>) -> Result<()> {
    let mops = Path::new("mops.toml");
    let mut doc = if mops.exists() {
        let str = std::fs::read_to_string(mops)?;
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
    std::fs::write(mops, doc.to_string())?;
    update_mops_lock(agent).await?;
    Ok(())
}
async fn update_mops_lock(agent: &Agent) -> Result<()> {
    let lock = Path::new("mops.lock");
    let doc = if lock.exists() {
        let str = std::fs::read_to_string(lock)?;
        let doc = str.parse::<ImDocument<_>>()?;
        toml_edit::de::from_document::<Packages>(doc)?
    } else {
        Packages::default()
    };
    let mut map: BTreeMap<_, _> = doc
        .package
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect();
    let mops = parse_mops_toml()?;
    let service = mops::Service(mops::CANISTER_ID, agent);
    for m in mops.into_iter() {
        match m {
            Mops::Mops { name, version } => {
                if map.contains_key(&name) {
                    println!("skipping {name}");
                    continue;
                }
                let pkg = service
                    .get_package_details(&name, &version)
                    .await?
                    .into_result()
                    .map_err(Error::msg)?;
                let source = pkg.publication.storage.to_string();
                let dependencies = pkg
                    .config
                    .dependencies
                    .into_iter()
                    .map(|d| {
                        let name = d.name;
                        /*if d.version.is_empty() {
                            Mops::Repo{ name, repo: d.repo }
                        } else {
                            Mops::Mops{ name, version: d.version }
                        };*/
                        name
                    })
                    .collect();
                let pkg = Package {
                    name: name.clone(),
                    version: Some(version),
                    source,
                    dependencies,
                };
                map.insert(name, pkg);
            }
            _ => (),
        }
    }
    let mut res = DocumentMut::new();
    let mut array = toml_edit::ArrayOfTables::new();
    for p in map.values() {
        let d = toml_edit::ser::to_document(p)?;
        array.push(d.as_table().clone());
    }
    res.insert("package", toml_edit::Item::ArrayOfTables(array));
    println!("{}", res.to_string());
    Ok(())
}
#[derive(Debug, Serialize, Deserialize)]
enum Mops {
    Mops { name: String, version: String },
    Repo { name: String, repo: String },
    Local { name: String, path: String },
}
fn parse_mops_toml() -> Result<Vec<Mops>> {
    let mops = Path::new("mops.toml");
    let str = std::fs::read_to_string(mops)?;
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
