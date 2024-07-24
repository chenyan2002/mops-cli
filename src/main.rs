use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod build;
mod mops;
mod storage;
mod utils;

#[derive(Parser)]
enum ClapCommand {
    /// Build Motoko project
    Build(BuildArg),
}
#[derive(Parser)]
struct BuildArg {
    /// The path to the main Motoko file
    main: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cmd = ClapCommand::parse();
    match cmd {
        ClapCommand::Build(args) => {
            let main = args.main.unwrap_or_else(|| PathBuf::from("main.mo"));
            build::build(&main)?;
        }
    }
    Ok(())
}
