use crate::env::Env;
use crate::toml::{download_packages_from_lock, generate_moc_args, update_mops_toml};
use crate::utils::{create_spinner_bar, exec};
use anyhow::{anyhow, Context, Result};
use candid::Principal;
use console::style;
use ic_agent::Agent;
use indicatif::HumanDuration;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub async fn build(agent: &Agent, env: &Env, args: crate::BuildArg) -> Result<()> {
    let main_file = if let Some(file) = args.main {
        file
    } else {
        crate::env::guess_main_file()?
    };
    let start = Instant::now();
    if !args.lock {
        let imports = get_imports(&main_file, env, args.print_source_on_error)?;
        update_mops_toml(agent, env, imports).await?;
        download_packages_from_lock(agent, env).await?;
    }
    let lock_time = start.elapsed();
    let pkgs = generate_moc_args(env)?;
    let msg = format!("{:>12} {}", style("Compiling").cyan(), main_file.display());
    let bar = create_spinner_bar(msg);
    let mut moc = env.get_moc();
    moc.arg(&main_file).args(pkgs);
    let output = env.get_target_build_path(&args.name, &main_file);
    moc.arg("-o").arg(output);
    if args.print_source_on_error {
        moc.arg("--print-source-on-error");
    }
    if !args.extra_args.is_empty() {
        for arg in args.extra_args {
            moc.arg(arg);
        }
    } else {
        moc.arg("--release")
            .arg("--idl")
            .arg("--stable-types")
            .arg("--public-metadata")
            .arg("candid:service");
    }
    exec(moc, false, Some(&bar))?;
    bar.finish_and_clear();
    let mut msg = format!(
        "{:>12} {} in {}",
        style("Compiled").green().bold(),
        main_file.display(),
        HumanDuration(start.elapsed())
    );
    if !args.lock {
        msg.push_str(&format!(
            " ({} to analyze dependencies)",
            HumanDuration(lock_time)
        ));
    }
    println!("{msg}");
    Ok(())
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum MotokoImport {
    Canister(String),
    Ic(Principal),
    Lib(String),
    Local(PathBuf),
}
fn get_imports(main_path: &Path, env: &Env, display_src: bool) -> Result<BTreeSet<MotokoImport>> {
    fn get_imports_recursive(
        env: &Env,
        file: &Path,
        display_src: bool,
        result: &mut BTreeSet<MotokoImport>,
    ) -> Result<()> {
        if result.contains(&MotokoImport::Local(file.to_path_buf())) {
            return Ok(());
        }
        result.insert(MotokoImport::Local(file.to_path_buf()));
        let mut command = env.get_moc();
        command.arg("--print-deps").arg(file);
        if display_src {
            command.arg("--print-source-on-error");
        }
        let output = exec(command, true, None)?;
        for line in output.lines() {
            let import = MotokoImport::try_from(line).context("Failed to parse import.")?;
            match import {
                MotokoImport::Local(path) => {
                    get_imports_recursive(env, path.as_path(), display_src, result)?;
                }
                _ => {
                    result.insert(import);
                }
            }
        }
        Ok(())
    }
    let mut result = BTreeSet::new();
    get_imports_recursive(env, main_path, display_src, &mut result)?;
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
                    "ic:" => MotokoImport::Ic(
                        Principal::from_text(name)
                            .context(format!("Fail to parse import {url}"))?,
                    ),
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
