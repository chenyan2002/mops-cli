use crate::toml::update_mops_toml;
use crate::utils::{create_bar, get_moc};
use crate::{mops, storage};
use anyhow::{anyhow, Context, Error, Result};
use candid::Principal;
use futures::{future::try_join_all, try_join};
use ic_agent::Agent;
use indicatif::MultiProgress;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn build(agent: &Agent, main_file: &Path) -> Result<()> {
    let bars = Rc::new(MultiProgress::new());
    let imports = get_imports(main_file)?;
    let libs: Vec<_> = imports
        .iter()
        .filter_map(|import| {
            if let MotokoImport::Lib(lib) = import {
                Some(lib)
            } else {
                None
            }
        })
        .collect();
    update_mops_toml(agent, libs).await?;
    /*
    let service = Rc::new(mops::Service(mops::CANISTER_ID, agent));
    let mut futures = Vec::new();
    for lib in libs {
        futures.push(download_package(
            lib.to_string(),
            None,
            service.clone(),
            bars.clone(),
        ));
    }
    try_join_all(futures).await?;
    */
    Ok(())
}

async fn download_package(
    lib: String,
    version: Option<String>,
    service: Rc<mops::Service<'_>>,
    bars: Rc<MultiProgress>,
) -> Result<()> {
    let version = match version {
        Some(version) => version,
        None => service
            .get_highest_version(&lib)
            .await?
            .into_result()
            .map_err(Error::msg)?,
    };
    let bar = create_bar(&bars, format!("Downloading {} {}", lib, version));
    let (ids, pkg) = try_join!(
        service.get_file_ids(&lib, &version),
        service.get_package_details(&lib, &version)
    )?;
    let ids = ids.into_result().map_err(Error::msg)?;
    let pkg = pkg.into_result().map_err(Error::msg)?;
    let mut futures = Vec::new();
    let storage = Rc::new(storage::Service(pkg.publication.storage, service.1));
    for id in ids {
        futures.push(download_file(id, storage.clone()));
    }
    try_join_all(futures).await?;
    bar.finish_with_message(format!("Downloaded {} {}", lib, version));
    Ok(())
}
async fn download_file(id: String, storage: Rc<storage::Service<'_>>) -> Result<(String, Vec<u8>)> {
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
    //println!("{} {}", meta.path, blob.len());
    Ok((meta.path, blob))
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum MotokoImport {
    Canister(String),
    Ic(Principal),
    Lib(String),
    Local(PathBuf),
}
fn get_imports(main_path: &Path) -> Result<BTreeSet<MotokoImport>> {
    fn get_imports_recursive(file: &Path, result: &mut BTreeSet<MotokoImport>) -> Result<()> {
        if result.contains(&MotokoImport::Local(file.to_path_buf())) {
            return Ok(());
        }
        result.insert(MotokoImport::Local(file.to_path_buf()));
        let mut command = get_moc()?;
        let command = command.arg("--print-deps").arg(file);
        let output = command
            .output()
            .with_context(|| format!("Error executing {:#?}", command))?;
        if !output.status.success() {
            return Err(anyhow!(
                "Failed to get imports from {}: {}",
                file.display(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let output = String::from_utf8_lossy(&output.stdout);

        for line in output.lines() {
            let import = MotokoImport::try_from(line).context("Failed to parse import.")?;
            match import {
                MotokoImport::Local(path) => {
                    get_imports_recursive(path.as_path(), result)?;
                }
                _ => {
                    result.insert(import);
                }
            }
        }
        Ok(())
    }
    let mut result = BTreeSet::new();
    get_imports_recursive(main_path, &mut result)?;
    Ok(result)
}

impl TryFrom<&str> for MotokoImport {
    type Error = anyhow::Error;

    fn try_from(line: &str) -> Result<Self> {
        let (url, fullpath) = match line.find(' ') {
            Some(index) => {
                if index >= line.len() - 1 {
                    return Err(anyhow!("Unknown import {}", line));
                }
                let (url, fullpath) = line.split_at(index + 1);
                (url.trim_end(), Some(fullpath))
            }
            None => (line, None),
        };
        let import = match url.find(':') {
            Some(index) => {
                if index >= line.len() - 1 {
                    return Err(anyhow!("Unknown import {}", url));
                }
                let (prefix, name) = url.split_at(index + 1);
                match prefix {
                    "canister:" => MotokoImport::Canister(name.to_owned()),
                    "ic:" => MotokoImport::Ic(Principal::from_text(name)?),
                    "mo:" => match name.split_once('/') {
                        Some((lib, _)) => MotokoImport::Lib(lib.to_owned()),
                        None => MotokoImport::Lib(name.to_owned()),
                    },
                    _ => {
                        return Err(anyhow!("Unknown import {}", url));
                    }
                }
            }
            None => match fullpath {
                Some(fullpath) => {
                    let path = PathBuf::from(fullpath);
                    if !path.is_file() {
                        return Err(anyhow!("Cannot find import file {}", path.display()));
                    };
                    MotokoImport::Local(path)
                }
                None => {
                    return Err(anyhow!("Cannot resolve local import {}", url));
                }
            },
        };
        Ok(import)
    }
}
