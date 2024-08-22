use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;

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
