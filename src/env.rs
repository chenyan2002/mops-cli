use crate::github::{get_latest_release_info, get_latest_release_version};
use crate::utils::create_spinner_bar;
use anyhow::{anyhow, Result};
use console::style;
use flate2::read::GzDecoder;
use indicatif::HumanBytes;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

pub struct Env {
    pub cache_dir: PathBuf,
    pub project_root: PathBuf,
    pub toolchain: BTreeMap<String, String>,
}
impl Env {
    pub async fn new(cache_dir: &Option<PathBuf>) -> Result<Self> {
        let cache_dir = get_cache_dir(cache_dir)?;
        let project_root = find_project_root()?;
        let mut res = Self {
            cache_dir,
            project_root,
            toolchain: BTreeMap::new(),
        };
        res.get_toolchain()?;
        res.update_moc(false).await?;
        if !res.get_fmt_path().exists() {
            download_fmt(&res, None).await?;
        }
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
    pub fn get_moc_path(&self) -> PathBuf {
        self.get_binary_path().join("moc")
    }
    pub fn get_moc(&self) -> Command {
        Command::new(self.get_moc_path())
    }
    pub fn get_moc_version(&self) -> Result<String> {
        let mut cmd = self.get_moc();
        cmd.arg("--version");
        let version = crate::utils::exec(cmd, true, None)?;
        version
            .split_whitespace()
            .nth(2)
            .map(|x| x.to_owned())
            .ok_or_else(|| {
                anyhow!("Cannot get moc version, please check if moc is installed correctly.")
            })
    }
    pub async fn update_moc(&self, check_latest: bool) -> Result<()> {
        let expect_ver = match self.toolchain.get("moc") {
            Some(v) => Some(v.clone()),
            None => {
                if check_latest {
                    get_latest_release_version("dfinity/motoko").await.ok()
                } else {
                    None
                }
            }
        };
        let needs_update = if self.get_moc_path().exists() {
            if let Ok(cur_ver) = &self.get_moc_version() {
                expect_ver.as_ref().is_some_and(|v| v != cur_ver)
            } else {
                true
            }
        } else {
            true
        };
        if needs_update {
            download_moc(self, expect_ver).await?;
        }
        Ok(())
    }
    pub fn get_fmt_path(&self) -> PathBuf {
        self.get_binary_path().join("mo-fmt")
    }
    pub fn get_fmt(&self) -> Command {
        Command::new(self.get_fmt_path())
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
    pub async fn check_version(&self, bin: &str, repo: &str) -> Option<()> {
        let path = self.get_binary_path().join(bin);
        if path.exists() {
            let mut cmd = Command::new(path);
            cmd.arg("--version");
            let version = crate::utils::exec(cmd, true, None)
                .ok()?
                .trim()
                .trim()
                .to_owned();
            let tag = get_latest_release_info(repo).await.ok()?.tag_name;
            let tag = if let Some(tag) = tag.strip_prefix('v') {
                tag
            } else {
                &tag
            };
            if version.contains(tag) {
                println!("{version} is up to date.");
                Some(())
            } else {
                println!("{version}\nLatest release: {tag}");
                None
            }
        } else {
            None
        }
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

pub async fn download_moc(env: &Env, expect_ver: Option<String>) -> Result<()> {
    let platform = if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        anyhow::bail!("Unsupported platform");
    };
    let tag = if let Some(v) = expect_ver {
        v
    } else {
        get_latest_release_version("dfinity/motoko").await?
    };
    let url = format!("https://github.com/dfinity/motoko/releases/download/{tag}/motoko-{platform}-x86_64-{tag}.tar.gz");
    download_release(env, "moc", &url, &tag).await?;
    Ok(())
}
pub async fn download_fmt(env: &Env, expect_ver: Option<String>) -> Result<()> {
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        anyhow::bail!("Unsupported platform");
    };
    let tag = if let Some(v) = expect_ver {
        v
    } else {
        get_latest_release_version("dfinity/prettier-plugin-motoko").await?
    };
    let url = format!("https://github.com/dfinity/prettier-plugin-motoko/releases/download/v{tag}/mo-fmt-{platform}.tar.gz");
    download_release(env, "mo-fmt", &url, &tag).await?;
    Ok(())
}

async fn download_release(env: &Env, name: &str, url: &str, ver: &str) -> Result<()> {
    use std::io::Write;
    let bar = create_spinner_bar(format!("Downloading {name} {ver}"));
    let response = reqwest::get(url).await?;
    let gz_file = env
        .get_binary_path()
        .join(format!("{}-{}.tar.gz", name, ver));
    fs::create_dir_all(gz_file.parent().unwrap())?;
    let mut file = File::create(&gz_file)?;
    let content = response.bytes().await?;
    file.write_all(&content)?;
    bar.set_message(format!("Decompressing {name} {ver}"));
    let gz = File::open(&gz_file)?;
    let tar = GzDecoder::new(gz);
    let mut archive = Archive::new(tar);
    archive.unpack(env.get_binary_path())?;
    fs::remove_file(&gz_file)?;
    bar.set_message(format!(
        "{:>12} {name} {ver}",
        style("Installed").green().bold()
    ));
    bar.finish();
    Ok(())
}
