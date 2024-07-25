use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
    pub repo: String,
    pub tag: String,
    pub commit: String,
}

/// Parse github url as specified in `https://docs.mops.one/mops.toml`
pub fn parse_github_url(url: &str) -> Result<RepoInfo> {
    // https://github.com/icdevsorg/candy_library#v0.3.0@907a4e7363aac6c6a4e114ebc73e3d3f21e138af
    // or https://github.com/chenyan2002/motoko-splay.git
    let url = url
        .strip_prefix("https://github.com/")
        .ok_or_else(|| anyhow::anyhow!("invalid url"))?;
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("invalid url"));
    }
    let owner = parts[0];
    let mut repo_part = parts[1];
    if repo_part.ends_with(".git") {
        repo_part = repo_part.strip_suffix(".git").unwrap();
    }
    let repo_parts: Vec<&str> = repo_part.split('#').collect();
    let repo = format!("{}/{}", owner, repo_parts[0]);
    let mut tag = None;
    let mut commit = None;
    if repo_parts.len() > 1 {
        let tag_commit_parts: Vec<&str> = repo_parts[1].split('@').collect();
        tag = Some(tag_commit_parts[0].to_string());
        if tag_commit_parts.len() > 1 {
            commit = Some(tag_commit_parts[1].to_string());
        }
    }
    Ok(RepoInfo {
        repo,
        tag: tag.unwrap_or_else(|| "master".to_string()),
        commit: commit.unwrap_or_else(|| "HEAD".to_string()),
    })
}
