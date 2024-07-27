use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod build;
mod github;
mod mops;
mod storage;
mod toml;
mod utils;

#[derive(Parser)]
enum ClapCommand {
    /// Build Motoko project
    Build(BuildArg),
}
#[derive(Parser)]
pub struct BuildArg {
    /// The path to the main Motoko file
    pub main: Option<PathBuf>,
    #[arg(short, long)]
    /// Directory to store external dependencies
    pub cache_dir: Option<PathBuf>,
    #[arg(long)]
    /// Lock the dependencies
    pub lock: bool,
}

fn main() -> Result<()> {
    let cmd = ClapCommand::parse();
    let agent = ic_agent::Agent::builder()
        .with_url("https://icp0.io")
        .build()?;
    match cmd {
        ClapCommand::Build(args) => {
            build::build(&agent, args)?;
        }
    }
    Ok(())
}
