use crate::binary_cache::*;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct Env {
    pub cache_dir: PathBuf,
    pub project_root: PathBuf,
    pub toolchain: BTreeMap<String, String>,
    pub binary: BTreeMap<String, Box<dyn Binary>>,
}
impl Env {
    pub async fn new(cache_dir: &Option<PathBuf>) -> Result<Self> {
        let cache_dir = get_cache_dir(cache_dir)?;
        let project_root = find_project_root()?;
        let mut res = Self {
            cache_dir,
            project_root,
            toolchain: BTreeMap::new(),
            binary: BTreeMap::new(),
        };
        res.get_toolchain()?;
        res.binary.insert(
            "moc".to_owned(),
            Box::new(Moc {
                binary_path: res.get_binary_path(),
                expect_version: res.toolchain.get("moc").cloned(),
            }),
        );
        res.binary.insert(
            "mo-fmt".to_owned(),
            Box::new(Fmt {
                binary_path: res.get_binary_path(),
                expect_version: res.toolchain.get("mo-fmt").cloned(),
            }),
        );
        res.binary["moc"].update_binary(false).await?;
        res.binary["mo-fmt"].update_binary(false).await?;
        Ok(res)
    }
    pub fn get_mops_toml_path(&self) -> PathBuf {
        self.project_root.join("mops.toml")
    }
    pub fn get_mops_lock_path(&self) -> PathBuf {
        self.project_root.join("mops.lock")
    }
    pub fn get_target_path(&self) -> PathBuf {
        self.project_root.join("target")
    }
    pub fn get_target_idl_path(&self) -> PathBuf {
        self.get_target_path().join("idl")
    }
    pub fn get_target_build_path(&self, name: &Option<String>, main_file: &PathBuf) -> PathBuf {
        let name = name
            .clone()
            .unwrap_or_else(|| guess_build_name(main_file).unwrap_or("wasm".to_string()));
        let filename = format!("{name}.wasm");
        self.get_target_path().join(name).join(filename)
    }
    pub fn get_binary_path(&self) -> PathBuf {
        self.cache_dir.join("bin")
    }
    fn get_toolchain(&mut self) -> Result<()> {
        let toml = self.get_mops_toml_path();
        if toml.exists() {
            let toml = std::fs::read_to_string(toml)?;
            let toml = toml.parse::<toml_edit::ImDocument<_>>()?;
            if let Some(toolchain) = toml.get("toolchain") {
                if let Some(toolchain) = toolchain.as_table() {
                    for (k, v) in toolchain {
                        if let Some(v) = v.as_str() {
                            self.toolchain.insert(k.to_owned(), v.to_owned());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn find_project_root() -> Result<PathBuf> {
    let mut path = std::env::current_dir()?;
    loop {
        if path.join("mops.toml").exists() {
            return Ok(path);
        }
        if !path.pop() {
            break;
        }
    }
    Ok(std::env::current_dir()?)
}
pub fn get_cache_dir(base_path: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(dir) = base_path {
        Ok(PathBuf::from(dir))
    } else if let Ok(home) = std::env::var("HOME") {
        Ok(PathBuf::from(home).join(".mops"))
    } else {
        return Err(anyhow!(
            "Cannot find home directory, use --cache_dir to specify the cache directory."
        ));
    }
}
pub fn guess_main_file() -> Result<PathBuf> {
    if Path::new("main.mo").exists() {
        Ok(PathBuf::from("main.mo"))
    } else if Path::new("Main.mo").exists() {
        Ok(PathBuf::from("Main.mo"))
    } else {
        Err(anyhow!(
            "Cannot find main.mo or Main.mo, please specify the main file."
        ))
    }
}
fn guess_build_name(main_file: &PathBuf) -> Option<String> {
    let main_file = std::fs::canonicalize(main_file).ok()?;
    let stem = main_file.file_stem()?;
    Some(if stem == "main" || stem == "Main" {
        main_file
            .parent()?
            .components()
            .last()?
            .as_os_str()
            .to_str()?
            .to_owned()
    } else {
        stem.to_str()?.to_owned()
    })
}
