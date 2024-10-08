use crate::github::get_latest_release_version;
use crate::utils::create_spinner_bar;
use anyhow::Result;
use async_trait::async_trait;
use console::style;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Command;
use tar::Archive;

#[async_trait]
pub trait Binary: Send + Sync {
    fn name(&self) -> &str;
    fn repo(&self) -> &str;
    fn get_path(&self) -> PathBuf;
    fn get_cmd(&self) -> Command {
        Command::new(self.get_path())
    }
    fn get_expect_version(&self) -> &Option<String>;
    fn get_version(&self) -> Result<String>;
    async fn download_binary(&self, version: String) -> Result<()>;
    async fn update_binary(&self, need_latest: bool) -> Result<()> {
        let cur_ver = &self.get_version().unwrap_or_default();
        let need_ver = if let Some(exp_ver) = self.get_expect_version() {
            if cur_ver != exp_ver {
                Some(exp_ver.clone())
            } else {
                if need_latest {
                    let latest = get_latest_release_version(self.repo()).await?;
                    println!(
                        "Latest {} is {}, but {} is pinned to {} in mops.toml",
                        self.name(),
                        latest,
                        self.name(),
                        exp_ver
                    );
                    return Ok(());
                }
                None
            }
        } else if !need_latest && !cur_ver.is_empty() {
            None
        } else {
            let latest = get_latest_release_version(self.repo()).await?;
            if *cur_ver != latest {
                Some(latest)
            } else {
                None
            }
        };
        if let Some(ver) = need_ver {
            self.download_binary(ver).await?;
        } else if need_latest {
            println!("{} is already up-to-date", self.name());
        }
        Ok(())
    }
}

pub struct Moc {
    pub binary_path: PathBuf,
    pub expect_version: Option<String>,
}
#[async_trait]
impl Binary for Moc {
    fn name(&self) -> &str {
        "moc"
    }
    fn repo(&self) -> &str {
        "dfinity/motoko"
    }
    fn get_path(&self) -> PathBuf {
        self.binary_path.join(self.name())
    }
    fn get_expect_version(&self) -> &Option<String> {
        &self.expect_version
    }
    fn get_version(&self) -> Result<String> {
        get_binary_version(self, 2)
    }
    async fn download_binary(&self, ver: String) -> Result<()> {
        let platform = if cfg!(target_os = "macos") {
            "Darwin"
        } else if cfg!(target_os = "linux") {
            "Linux"
        } else {
            anyhow::bail!("Unsupported platform");
        };
        let url = format!("https://github.com/dfinity/motoko/releases/download/{ver}/motoko-{platform}-x86_64-{ver}.tar.gz");
        download_release(self, &url, &ver).await?;
        Ok(())
    }
}
pub struct Fmt {
    pub binary_path: PathBuf,
    pub expect_version: Option<String>,
}
#[async_trait]
impl Binary for Fmt {
    fn name(&self) -> &str {
        "mo-fmt"
    }
    fn repo(&self) -> &str {
        "dfinity/prettier-plugin-motoko"
    }
    fn get_path(&self) -> PathBuf {
        self.binary_path.join(self.name())
    }
    fn get_expect_version(&self) -> &Option<String> {
        &self.expect_version
    }
    fn get_version(&self) -> Result<String> {
        get_binary_version(self, 1)
    }
    async fn download_binary(&self, ver: String) -> Result<()> {
        let platform = if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            anyhow::bail!("Unsupported platform");
        };
        let url = format!("https://github.com/dfinity/prettier-plugin-motoko/releases/download/v{ver}/mo-fmt-{platform}.tar.gz");
        download_release(self, &url, &ver).await?;
        Ok(())
    }
}

fn get_binary_version(bin: &dyn Binary, pos: usize) -> Result<String> {
    let mut cmd = bin.get_cmd();
    cmd.arg("--version");
    let version = crate::utils::exec(cmd, true, None)?;
    version
        .split_whitespace()
        .nth(pos)
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("failed to get version for {}", bin.name()))
}
async fn download_release<T: Binary>(bin: &T, url: &str, ver: &str) -> Result<()> {
    use std::io::Write;
    let name = format!("{} {}", bin.name(), ver);
    let bar = create_spinner_bar(format!("Downloading {name}"));
    let base_path = bin.get_path().parent().unwrap().to_path_buf();
    let gz_file = base_path.join(format!("{}-{}.tar.gz", bin.name(), ver));
    fs::create_dir_all(&base_path)?;
    let mut file = File::create(&gz_file)?;
    let response = reqwest::get(url).await?;
    let content = response.bytes().await?;
    file.write_all(&content)?;
    bar.set_message(format!("Decompressing {name}"));
    let gz = File::open(&gz_file)?;
    let tar = GzDecoder::new(gz);
    let mut archive = Archive::new(tar);
    archive.unpack(base_path)?;
    fs::remove_file(&gz_file)?;
    bar.set_message(format!("{:>12} {name}", style("Installed").green().bold()));
    bar.finish();
    Ok(())
}
