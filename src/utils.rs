use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;

pub fn get_moc() -> Result<Command> {
    let dfx_cache = Command::new("dfx").args(["cache", "show"]).output()?.stdout;
    let dfx_cache_path = String::from_utf8_lossy(&dfx_cache).trim().to_string();
    let cmd = Command::new(format!("{}/moc", dfx_cache_path));
    Ok(cmd)
}

pub fn exec(mut cmd: Command, bar: &ProgressBar) -> Result<()> {
    let output = cmd
        .output()
        .with_context(|| format!("Error executing {:#?}", cmd))?;
    if !output.stderr.is_empty() {
        bar.println(String::from_utf8_lossy(&output.stderr));
    }
    if !output.stdout.is_empty() {
        bar.println(String::from_utf8_lossy(&output.stdout));
    }
    if !output.status.success() {
        return Err(anyhow!("Exit with code {}", output.status));
    }
    Ok(())
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
