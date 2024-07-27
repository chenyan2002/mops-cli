# Mops CLI

A Rust client implementation of [the Motoko package manger](https://mops.one/).

## Differences from the [node client](https://github.com/ZenVoich/mops/tree/main/cli)

* `mops.toml` can be auto-generated from `main.mo` if the packages are all on mops.
* Similar to `cargo build`, `mops build` generates a `mops.lock` file that records the precise dependencies of the project. Note that the lock file format is different from the node client.
* `mops build` can automatically download external dependencies specified in `mops.lock`, without the need to run `mops install`.
* The downloaded packages are stored globally at `$HOME/.mops`, similar to cargo.
* Overall, users can run `mops build main.mo` directly without any setup.

## Pending issues

* Resolving package versions. Currently, we choose the largest version when package names collide, and errors out when we cannot decide on the version of a package. We need compiler support to allow the same package name to apply to different modules, and follow semantics versioning.
* Removing a dependency doesn't remove the entries in `mops.lock`

 
