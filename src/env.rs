use crate::github::get_latest_release_tag;
use crate::utils::create_spinner_bar;
use anyhow::{anyhow, Result};
use console::style;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

pub struct Env {
    pub cache_dir: PathBuf,
    pub project_root: PathBuf,
}
impl Env {
    pub async fn new(cache_dir: &Option<PathBuf>) -> Result<Self> {
        let cache_dir = get_cache_dir(cache_dir)?;
        let project_root = find_project_root()?;
        let res = Self {
            cache_dir,
            project_root,
        };
        if !res.get_moc_path().exists() {
            download_moc(&res.cache_dir).await?;
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
        let res = self.get_target_path().join(name).join(filename);
        fs::create_dir_all(res.parent().unwrap()).unwrap();
        res
    }
    pub fn get_moc_path(&self) -> PathBuf {
        self.cache_dir.join("bin/moc")
    }
    pub fn get_moc(&self) -> Command {
        Command::new(self.get_moc_path())
    }
    pub fn get_fmt_path(&self) -> PathBuf {
        self.cache_dir.join("bin/mo-fmt")
    }
    pub fn get_fmt(&self) -> Command {
        Command::new(self.get_fmt_path())
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

pub async fn download_moc(base_path: &Path) -> Result<()> {
    let platform = if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        anyhow::bail!("Unsupported platform");
    };
    let url = "https://github.com/dfinity/motoko/releases/download/{tag}/motoko-{platform}-x86_64-{tag}.tar.gz";
    download_release(base_path, "moc", "dfinity/motoko", url, platform).await?;
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        anyhow::bail!("Unsupported platform");
    };
    let url = "https://github.com/dfinity/prettier-plugin-motoko/releases/download/{tag}/mo-fmt-{platform}.tar.gz";
    download_release(
        base_path,
        "mo-fmt",
        "dfinity/prettier-plugin-motoko",
        url,
        platform,
    )
    .await?;
    Ok(())
}

async fn download_release(
    base_path: &Path,
    name: &str,
    repo: &str,
    url: &str,
    platform: &str,
) -> Result<()> {
    use std::io::Write;
    let bar = create_spinner_bar(format!("Downloading {}", repo));
    let tag = get_latest_release_tag(repo).await?;
    bar.set_message(format!("Downloading {name} {tag}"));
    let url = url.replace("{tag}", &tag).replace("{platform}", platform);
    let response = reqwest::get(url).await?;
    let gz_file = base_path.join(format!("bin/{}-{}.tar.gz", name, tag));
    fs::create_dir_all(gz_file.parent().unwrap())?;
    let mut file = File::create(&gz_file)?;
    let content = response.bytes().await?;
    file.write_all(&content)?;
    bar.set_message(format!("Decompressing {name} {tag}"));
    let gz = File::open(&gz_file)?;
    let tar = GzDecoder::new(gz);
    let mut archive = Archive::new(tar);
    archive.unpack(base_path.join("bin"))?;
    fs::remove_file(&gz_file)?;
    bar.set_message(format!(
        "{:>12} {name} {tag}",
        style("Installed").green().bold()
    ));
    bar.finish();
    Ok(())
}
