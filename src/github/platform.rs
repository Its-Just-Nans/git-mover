//! Github Platform
use super::{GITHUB_API_HEADER, GITHUB_API_URL, GITHUB_API_VERSION, GITHUB_URL};
use crate::{
    errors::{GitMoverError, GitMoverErrorKind},
    github::repo::RepoGithub,
    platform::{Platform, PlatformType},
    utils::Repo,
};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use std::pin::Pin;
use urlencoding::encode;

/// Github Platform
#[derive(Default, Debug, Clone)]
pub struct GithubPlatform {
    /// Github username
    username: String,

    /// Github token
    token: String,

    /// Reqwest client
    client: reqwest::Client,
}

impl GithubPlatform {
    /// Create a new GithubPlatform
    pub(crate) fn new(username: String, token: String) -> Self {
        Self {
            username,
            token,
            client: reqwest::Client::new(),
        }
    }
}

impl Platform for GithubPlatform {
    fn get_remote_url(&self) -> &str {
        GITHUB_URL
    }

    fn get_username(&self) -> &str {
        &self.username
    }

    fn get_type(&self) -> PlatformType {
        PlatformType::Github
    }

    fn create_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let repo = repo.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!("https://{GITHUB_API_URL}/user/repos");
            let request = client
                .post(&url)
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .header(ACCEPT, "application/vnd.github+json")
                .header(USER_AGENT, "reqwest")
                .header(GITHUB_API_HEADER, GITHUB_API_VERSION)
                .json(&repo)
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                let get_repo = match self.get_repo(repo.name.as_str()).await {
                    Ok(repo) => repo,
                    Err(e) => {
                        let text_error = format!("{} - {}", &text, e);
                        return Err(GitMoverError::new(GitMoverErrorKind::RepoCreation)
                            .with_platform(PlatformType::Github)
                            .with_text(&text_error));
                    }
                };
                if get_repo != repo {
                    return self.edit_repo(repo).await;
                }
            }
            Ok(())
        })
    }

    fn edit_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!(
                "https://{}/repos/{}/{}",
                GITHUB_API_URL,
                self.username,
                encode(&repo.name)
            );
            let request = client
                .patch(&url)
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .header(ACCEPT, "application/vnd.github+json")
                .header(USER_AGENT, "reqwest")
                .header(GITHUB_API_HEADER, GITHUB_API_VERSION)
                .json(&repo)
                .send();
            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::RepoEdition)
                    .with_platform(PlatformType::Github)
                    .with_text(&text));
            }
            Ok(())
        })
    }

    fn get_repo(
        &self,
        repo_name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Repo, GitMoverError>> + Send>> {
        let token = self.token.clone();
        let username = self.username.clone();
        let repo_name = repo_name.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!(
                "https://{}/repos/{}/{}",
                GITHUB_API_URL,
                username,
                encode(&repo_name)
            );
            let request = client
                .get(&url)
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .header(ACCEPT, "application/vnd.github+json")
                .header(USER_AGENT, "reqwest")
                .header(GITHUB_API_HEADER, GITHUB_API_VERSION)
                .send();
            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::GetRepo)
                    .with_platform(PlatformType::Github)
                    .with_text(&text));
            }
            let text = response.text().await?;
            let repo: RepoGithub = serde_json::from_str(&text)?;
            Ok(repo.into())
        })
    }

    fn get_all_repos(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<Repo>, GitMoverError>> + Send>> {
        let token = self.token.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let url = &format!("https://{GITHUB_API_URL}/user/repos");
            let mut need_request = true;
            let mut page: usize = 1;
            let mut all_repos = vec![];
            while need_request {
                let request = client
                    .get(url)
                    .query(&[
                        ("type", "owner"),
                        ("per_page", "100"),
                        ("page", &page.to_string()),
                    ])
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .header(ACCEPT, "application/vnd.github+json")
                    .header(USER_AGENT, "reqwest")
                    .header(GITHUB_API_HEADER, GITHUB_API_VERSION)
                    .send();
                let response = request.await?;
                if !response.status().is_success() {
                    let text = response.text().await?;
                    return Err(GitMoverError::new(GitMoverErrorKind::GetAllRepos)
                        .with_platform(PlatformType::Github)
                        .with_text(&text));
                }
                let text = response.text().await?;
                let repos: Vec<RepoGithub> = serde_json::from_str(&text)?;
                let repos: Vec<Repo> = repos.into_iter().map(|r| r.into()).collect();
                if repos.is_empty() {
                    need_request = false;
                }
                println!("Requested github (page {}): {}", page, repos.len());
                all_repos.extend(repos);
                page += 1;
            }
            Ok(all_repos)
        })
    }

    fn delete_repo(
        &self,
        repo_name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let name = repo_name.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!(
                "https://{}/repos/{}/{}",
                GITHUB_API_URL,
                self.username,
                encode(&name)
            );
            let request = client
                .delete(&url)
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .header(ACCEPT, "application/vnd.github+json")
                .header(USER_AGENT, "reqwest")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send();
            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::RepoDeletion)
                    .with_platform(PlatformType::Github)
                    .with_text(&text));
            }
            Ok(())
        })
    }
}
