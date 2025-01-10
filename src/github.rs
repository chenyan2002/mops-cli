use crate::utils::println;
use anyhow::Result;
use console::style;
use futures::future::try_join_all;
use indicatif::ProgressBar;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoInfo {
    pub repo: String,
    pub tag: String,
    pub commit: String,
    pub base_dir: String,
}

/// Parse github url as specified in `https://docs.mops.one/mops.toml`
pub async fn parse_github_url(url: &str) -> Result<RepoInfo> {
    // https://github.com/icdevsorg/candy_library/base_dir#v0.3.0@907a4e7363aac6c6a4e114ebc73e3d3f21e138af
    // or https://github.com/chenyan2002/motoko-splay.git
    let url = url
        .strip_prefix("https://github.com/")
        .ok_or_else(|| anyhow::anyhow!("invalid url"))?;
    let parts: Vec<&str> = url.splitn(3, '/').collect();
    let base_dir = match parts.len() {
        0 | 1 => return Err(anyhow::anyhow!("invalid url")),
        2 => "src".to_string(),
        3 => parts[2].to_string(),
        _ => unreachable!(),
    };
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
    if tag.is_none() {
        tag = Some(get_default_branch(&repo).await?);
    }
    if commit.is_none() {
        commit = Some(get_latest_commit(&repo, tag.as_ref().unwrap()).await?);
    }
    Ok(RepoInfo {
        repo,
        tag: tag.unwrap(),
        commit: commit.unwrap(),
        base_dir,
    })
}

pub async fn download_github_package(
    base_path: PathBuf,
    repo: RepoInfo,
    bar: Rc<ProgressBar>,
) -> Result<()> {
    let files = get_file_list(&repo).await?;
    let mut futures = Vec::new();
    for file in files {
        futures.push(download_file(base_path.clone(), repo.clone(), file));
    }
    try_join_all(futures).await?;
    fs::write(base_path.join(repo.get_done_file()), "")?;
    println(
        Some(&bar),
        "stdout",
        &format!(
            "{:>12} {}@{}",
            style("Downloaded").green().bold(),
            repo.repo,
            repo.tag
        ),
    );
    bar.inc(1);
    Ok(())
}

async fn download_file(base_path: PathBuf, repo: RepoInfo, file: String) -> Result<()> {
    let content = fetch_file(&repo, &file).await?;
    let path = base_path.join(file);
    let dir = path.parent().unwrap();
    std::fs::create_dir_all(dir)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub async fn fetch_file(repo: &RepoInfo, file: &str) -> Result<String> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        repo.repo, repo.commit, file
    );
    let body = github_request(&url).await?;
    if body.starts_with("404: Not Found") {
        return Err(anyhow::anyhow!("file not found"));
    }
    Ok(body)
}
async fn get_default_branch(repo: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct Branch {
        default_branch: String,
    }
    let url = format!("https://api.github.com/repos/{}", repo);
    let body = github_request(&url).await?;
    let response = serde_json::from_str::<Branch>(&body).map_err(|_| anyhow::anyhow!("{body}"))?;
    Ok(response.default_branch)
}

pub async fn get_latest_commit(repo: &str, tag: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct Commit {
        sha: String,
    }
    let url = format!("https://api.github.com/repos/{}/commits/{}", repo, tag);
    let body = github_request(&url).await?;
    let response = serde_json::from_str::<Commit>(&body).map_err(|_| anyhow::anyhow!("{body}"))?;
    Ok(response.sha)
}
#[derive(Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}
#[derive(Deserialize)]
pub struct Asset {
    pub size: u64,
    pub browser_download_url: String,
}
async fn get_latest_release_info(repo: &str) -> Result<ReleaseInfo> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let body = github_request(&url).await?;
    let response =
        serde_json::from_str::<ReleaseInfo>(&body).map_err(|_| anyhow::anyhow!("{body}"))?;
    Ok(response)
}
pub async fn get_latest_tag(repo: &str) -> Result<String> {
    let url = format!("https://api.github.com/repos/{}/tags", repo);
    let body = github_request(&url).await?;
    let response: Vec<serde_json::Value> =
        serde_json::from_str(&body).map_err(|_| anyhow::anyhow!("{body}"))?;
    if let Some(tag) = response.first() {
        if let Some(tag_name) = tag.get("name").and_then(|v| v.as_str()) {
            Ok(tag_name.to_string())
        } else {
            Err(anyhow::anyhow!("Tag name not found"))
        }
    } else {
        Err(anyhow::anyhow!("No tags found in the repo {repo}"))
    }
}
async fn get_file_list(repo: &RepoInfo) -> Result<Vec<String>> {
    #[derive(Deserialize)]
    struct Tree {
        tree: Vec<TreeItem>,
    }
    #[derive(Deserialize)]
    struct TreeItem {
        path: String,
        r#type: String,
    }
    let url = format!(
        "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
        repo.repo, repo.commit
    );
    let body = github_request(&url).await?;
    let tree = serde_json::from_str::<Tree>(&body).map_err(|_| anyhow::anyhow!("{body}"))?;
    Ok(tree
        .tree
        .into_iter()
        .filter(|item| {
            item.r#type == "blob"
                && item.path.starts_with(&repo.base_dir)
                && item.path.ends_with(".mo")
        })
        .map(|item| item.path)
        .collect())
}
async fn github_request(url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let mut request = client.get(url).header("User-Agent", "mops-cli");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let response = request.send().await?;
    let body = response.text().await?;
    Ok(body)
}
fn guess_version_from_tag(tag: &str) -> Option<Version> {
    let idx = tag.find(|c: char| c.is_ascii_digit())?;
    let maybe = &tag[idx..];
    maybe.parse::<Version>().ok()
}
pub async fn get_latest_release_version(repo: &str) -> Result<String> {
    let tag = get_latest_release_info(repo).await?.tag_name;
    guess_version_from_tag(&tag)
        .map(|v| v.to_string())
        .ok_or_else(|| anyhow::anyhow!("invalid version"))
}
impl RepoInfo {
    pub fn get_done_file(&self) -> String {
        format!("DONE-{}", self.base_dir.replace('/', "-"))
    }
    pub fn guess_version(&self) -> Option<String> {
        guess_version_from_tag(&self.tag).map(|v| v.to_string())
    }
}
impl ReleaseInfo {
    #[allow(dead_code)]
    pub fn get_asset_size(&self, url: &str) -> Option<u64> {
        self.assets
            .iter()
            .find(|asset| asset.browser_download_url == url)
            .map(|asset| asset.size)
    }
}
