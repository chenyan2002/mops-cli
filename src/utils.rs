use crate::github::get_latest_release_tag;
use anyhow::{anyhow, Context, Result};
use console::style;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

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

pub fn get_moc(base_path: &Path) -> Result<Command> {
    let cmd = Command::new(format!("{}/bin/moc", base_path.display()));
    Ok(cmd)
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
pub async fn download_moc(base_path: &Path) -> Result<()> {
    let platform = if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        anyhow::bail!("Unsupported platform");
    };
    let url = "https://github.com/dfinity/motoko/releases/download/{tag}/motoko-{platform}-x86_64-{tag}.tar.gz";
    download_release(base_path, "moc", "dfinity/motoko", &url, &platform).await?;
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
        &url,
        &platform,
    )
    .await?;
    Ok(())
}

pub fn exec(mut cmd: Command, is_silence: bool, bar: Option<&ProgressBar>) -> Result<String> {
    let output = cmd
        .output()
        .with_context(|| format!("Error executing {:#?}", cmd))?;
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println(bar, "stderr", &stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !is_silence && !stdout.is_empty() {
        println(bar, "stdout", &stdout);
    }
    if !output.status.success() {
        return Err(anyhow!("Exit with code {}", output.status));
    }
    Ok(stdout.to_string())
}
pub fn println(bar: Option<&ProgressBar>, target: &str, msg: &str) {
    if bar.is_none() || bar.is_some_and(|bar| bar.is_hidden()) {
        if target == "stderr" {
            eprintln!("{msg}");
        } else {
            println!("{msg}");
        }
    } else {
        #[allow(clippy::unnecessary_unwrap)]
        bar.unwrap().println(msg);
    }
}

pub fn create_bar(len: usize) -> ProgressBar {
    ProgressBar::new(len as u64).with_style(
        ProgressStyle::with_template("{prefix:>12.cyan.bold} [{bar:57.green}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    )
}

pub fn create_spinner_bar(msg: impl Into<std::borrow::Cow<'static, str>>) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(std::time::Duration::from_millis(200));
    pb.set_message(msg);
    pb
}
