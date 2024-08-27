# Mops CLI

A Rust client implementation of [the Motoko package manger](https://mops.one/).

If you have many packages from github, you may get rate limited from github. You can create a [personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens), and put the token in the `GITHUB_TOKEN` environment variable to get a much higher limit.

## Conventions

* Cache directory: `~/.mops/`, stores the Motoko compiler binary and library dependencies downloaded from mops or github. It can be changed by using the `--cache-dir` flag.
* Project root directory: The first occurance of `mops.toml` from the current directory to its parent directories. If `mops.toml` is missing, the current directory is the root directory, and a `mops.toml` will be auto-generated.
* Main Motoko file: `mops-cli build <main_file>`. If `<main_file>` is omitted, will use `main.mo` or `Main.mo`.
* Build artifacts: Stored in `<root_directory>/target/<name>/<name>.wasm`, where `<name>` can be specified by `mops-cli build --name <name>`. If `--name` is omitted, `<name>` will be the filename of the main Motoko file. If the filename is `Main.mo` or `main.mo`, `<name>` will be the parent directory name. If anything fails, we use `wasm` as the default `<name>`.
* Compiler flags: `--release --idl --stable-types --public-metadata candid:service -o target/<name>/<name>.wasm --package <from_mops_lock>`. If extra arguments are passed via `mops-cli build -- <moc_args>`, the default flags will be dropped, except `-o and --package` flags. If `<moc_args>` contains `-o`, the default `-o` flag will be dropped.

## Differences from the [node client](https://github.com/ZenVoich/mops/tree/main/cli)

* `mops.toml` can be auto-generated from `main.mo` if the packages are all on mops.
* Similar to `cargo build`, `mops build` generates a `mops.lock` file that records the precise dependencies of the project. Note that the lock file format is different from the node client.
* `mops build` can automatically download external dependencies specified in `mops.lock`, without the need to run `mops install`.
* The downloaded packages are stored globally at `$HOME/.mops`, similar to cargo.
* Overall, users can run `mops build main.mo` directly without any setup.
* `import Backend "canister:backend"` can be configured in `mops.toml`,
```toml
[[canister]]
name = "backend"
canister_id = "ryjl3-tyaaa-aaaaa-aaaba-cai"
```

## Pending issues

* Resolving package versions. Currently, we choose the largest version when package names collide, and errors out when we cannot decide on the version of a package. We need compiler support to allow the same package name to apply to different modules, and follow semantic versioning. The base library also need to follow semantic versioning.
* Removing a dependency doesn't remove the entries in `mops.lock`

 
