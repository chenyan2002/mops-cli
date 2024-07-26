use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;

pub fn get_moc() -> Result<Command> {
    let dfx_cache = Command::new("dfx").args(["cache", "show"]).output()?.stdout;
    let dfx_cache_path = String::from_utf8_lossy(&dfx_cache).trim().to_string();
    let cmd = Command::new(format!("{}/moc", dfx_cache_path));
    Ok(cmd)
}

pub fn create_bar(len: u64) -> ProgressBar {
    ProgressBar::new(len).with_style(
        ProgressStyle::with_template("{prefix:>12.cyan.bold} [{bar:57.green}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    )
}
/*
pub fn create_bar(
    bars: &MultiProgress,
    msg: impl Into<std::borrow::Cow<'static, str>>,
) -> ProgressBar {
    let pb = bars.add(ProgressBar::new_spinner());
    pb.enable_steady_tick(std::time::Duration::from_millis(200));
    pb.set_message(msg);
    pb
}
*/
