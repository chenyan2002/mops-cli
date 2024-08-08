use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod build;
mod github;
mod mops;
mod storage;
mod toml;
mod utils;

use crate::utils::{download_moc, exec, get_cache_dir, get_moc};

#[derive(Parser)]
enum ClapCommand {
    /// Build Motoko project
    Build(BuildArg),
    /// Calls the Motoko compiler
    Moc(MocArg),
    /// Update the dependencies or the Motoko compiler
    Update(UpdateArg),
}
#[derive(Parser)]
struct UpdateArg {
    /// Directory to store external dependencies
    pub cache_dir: Option<PathBuf>,
    #[arg(short, long)]
    /// Download the latest Motoko compiler
    pub moc: bool,
}
#[derive(Parser)]
struct MocArg {
    /// Directory to store external dependencies
    pub cache_dir: Option<PathBuf>,
    #[clap(last = true)]
    /// Arguments passed to moc
    extra_args: Vec<String>,
}
#[derive(Parser)]
pub struct BuildArg {
    /// The path to the main Motoko file
    pub main: Option<PathBuf>,
    #[arg(short, long)]
    /// Directory to store external dependencies
    pub cache_dir: Option<PathBuf>,
    #[arg(short, long)]
    /// Output Wasm file path
    pub output: Option<String>,
    #[arg(long)]
    /// Lock the dependencies
    pub lock: bool,
    #[arg(short, long)]
    /// Display the source code for error messages
    pub print_source_on_error: bool,
    #[clap(last = true)]
    /// Extra arguments passed to moc. Default args are "--release --idl --stable-types --public-metadata candid:service". When extra arguments are provided, the default args are not included.
    extra_args: Vec<String>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let cmd = ClapCommand::parse();
    let agent = ic_agent::Agent::builder()
        .with_url("https://icp0.io")
        .build()?;
    match cmd {
        ClapCommand::Moc(args) => {
            let cache_dir = get_cache_dir(&args.cache_dir)?;
            let mut moc = get_moc(&cache_dir)?;
            moc.args(&args.extra_args);
            exec(moc, false, None)?;
        }
        ClapCommand::Build(args) => {
            build::build(&agent, args).await?;
        }
        ClapCommand::Update(args) => {
            let cache_dir = get_cache_dir(&args.cache_dir)?;
            if args.moc {
                let mut moc = get_moc(&cache_dir)?;
                moc.arg("--version");
                let version = exec(moc, true, None)?;
                let tag = github::get_latest_release_tag("dfinity/motoko").await?;
                println!("Current version: {version}Latest release: {tag}");
                download_moc(&cache_dir).await?;
            } else {
                unimplemented!();
            }
        }
    }
    Ok(())
}
