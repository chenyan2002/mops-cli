use crate::toml::{download_packages_from_lock, update_mops_toml};
use crate::utils::get_moc;
use anyhow::{anyhow, Context, Result};
use candid::Principal;
use ic_agent::Agent;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
pub async fn build(agent: &Agent, args: crate::BuildArg) -> Result<()> {
    let main_file = args.main.unwrap_or_else(|| PathBuf::from("main.mo"));
    let cache_dir = args.cache_dir.unwrap_or_else(|| PathBuf::from(".mops"));
    if !args.lock {
        let imports = get_imports(&main_file)?;
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
        download_packages_from_lock(agent, &cache_dir).await?;
    }
    Ok(())
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
