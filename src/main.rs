use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod binary_cache;
mod build;
mod env;
mod github;
mod mops;
mod storage;
mod toml;
mod utils;

use crate::utils::exec;
#[derive(Parser)]
struct Opts {
    #[arg(short, long)]
    /// Directory to cache external dependencies
    cache_dir: Option<PathBuf>,
    #[command(subcommand)]
    cmd: ClapCommand,
}

#[derive(Parser)]
enum ClapCommand {
    /// Build Motoko project
    Build(BuildArg),
    /// Calls the Motoko compiler
    Moc(MocArg),
    /// Update the dependencies or the Motoko compiler
    Update(UpdateArg),
    /// Motoko formatter
    Fmt(FmtArg),
}
#[derive(Parser)]
struct UpdateArg {
    #[arg(short, long)]
    /// Download the latest Motoko compiler
    pub moc: bool,
}
#[derive(Parser)]
struct FmtArg {
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    /// Arguments passed to mo-fmt. No need to add "--" before the arguments.
    extra_args: Vec<String>,
}
#[derive(Parser)]
struct MocArg {
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    /// Arguments passed to moc. No need to add "--" before the arguments.
    extra_args: Vec<String>,
}
#[derive(Parser)]
pub struct BuildArg {
    /// The path to the main Motoko file
    pub main: Option<PathBuf>,
    #[arg(short, long)]
    /// Output Wasm file path at target/<name>/. Can be overridden by specifying "-- -o output.wasm".
    pub name: Option<String>,
    #[arg(long)]
    /// Lock the dependencies
    pub lock: bool,
    #[arg(short, long)]
    /// Display the source code for error messages
    pub print_source_on_error: bool,
    #[clap(last = true)]
    /// Extra arguments passed to moc. Need to add "--" before the arguments. Default args are "--release --idl --stable-types --public-metadata candid:service". When extra arguments are provided, the default args are not included.
    extra_args: Vec<String>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let env = env::Env::new(&opts.cache_dir).await?;
    let agent = ic_agent::Agent::builder()
        .with_url("https://icp0.io")
        .build()?;
    match opts.cmd {
        ClapCommand::Moc(args) => {
            let mut moc = env.binary["moc"].get_cmd();
            moc.args(&args.extra_args);
            exec(moc, false, None)?;
        }
        ClapCommand::Build(args) => {
            build::build(&agent, &env, args).await?;
        }
        ClapCommand::Update(args) => {
            if args.moc {
                for bin in env.binary.values() {
                    bin.update_binary(true).await?;
                }
            } else {
                toml::update_packages_from_lock(&agent, &env).await?;
            }
        }
        ClapCommand::Fmt(args) => {
            let mut fmt = env.binary["mo-fmt"].get_cmd();
            fmt.args(&args.extra_args);
            exec(fmt, false, None)?;
        }
    }
    Ok(())
}
