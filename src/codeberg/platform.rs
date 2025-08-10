//! Codeberg platform implementation
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use std::pin::Pin;
use urlencoding::encode;

use super::{repo::CodebergRepo, CODEBERG_URL};
use crate::{
    errors::{GitMoverError, GitMoverErrorKind},
    platform::{Platform, PlatformType},
    utils::Repo,
};

/// Codeberg platform
#[derive(Default, Debug, Clone)]
pub struct CodebergPlatform {
    /// Codeberg username
    username: String,
    /// Codeberg token
    token: String,

    /// Reqwest client
    client: reqwest::Client,
}

impl CodebergPlatform {
    /// Create a new codeberg platform
    pub fn new(username: String, token: String) -> Self {
        Self {
            username,
            token,
            client: reqwest::Client::new(),
        }
    }
}

impl Platform for CodebergPlatform {
    fn get_remote_url(&self) -> &str {
        CODEBERG_URL
    }

    fn get_username(&self) -> &str {
        &self.username
    }

    fn get_type(&self) -> PlatformType {
        PlatformType::Codeberg
    }

    fn create_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let repo_name = repo.name.to_string();
        let description = repo.description.to_string();
        let private = repo.private;
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!("https://{CODEBERG_URL}/api/v1/user/repos");
            let json_body = CodebergRepo {
                name: repo_name.to_string(),
                description: description.to_string(),
                private,
                fork: false, // not uset
            };
            let request = client
                .post(url)
                .header(AUTHORIZATION, format!("token {token}"))
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .json(&json_body)
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                let get_repo = match self.get_repo(repo_name.as_str()).await {
                    Ok(repo) => repo,
                    Err(e) => {
                        let text_error = format!("{} - {}", &text, e);
                        return Err(GitMoverError::new(GitMoverErrorKind::RepoCreation)
                            .with_platform(PlatformType::Codeberg)
                            .with_text(&text_error));
                    }
                };
                let json_body_as_repo = json_body.clone().into();
                if get_repo != json_body_as_repo {
                    eprintln!(
                        "Repository already exists with different configuration {get_repo:?} {json_body_as_repo:?}"
                    );
                    return self.edit_repo(json_body_as_repo).await;
                } else {
                    return Ok(());
                }
            }

            Ok(())
        })
    }

    fn get_repo(
        &self,
        repo_name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Repo, GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let repo_name = repo_name.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!(
                "https://{}/api/v1/repos/{}/{}",
                CODEBERG_URL,
                self.get_username(),
                encode(&repo_name)
            );
            let request = client
                .get(&url)
                .header(AUTHORIZATION, format!("token {token}"))
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::GetRepo)
                    .with_platform(PlatformType::Codeberg)
                    .with_text(&text));
            }
            let repo: CodebergRepo = response.json().await?;
            Ok(repo.into())
        })
    }

    fn edit_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let repo = repo.clone();
        let token = self.token.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!(
                "https://{}/api/v1/repos/{}/{}",
                CODEBERG_URL,
                self.get_username(),
                encode(&repo.name)
            );
            let json_body = CodebergRepo {
                name: repo.name.to_string(),
                description: repo.description.to_string(),
                private: repo.private,
                fork: repo.fork, // not uset
            };
            let request = client
                .patch(url)
                .header(AUTHORIZATION, format!("token {token}"))
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .json(&json_body)
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::RepoEdition)
                    .with_platform(PlatformType::Codeberg)
                    .with_text(&text));
            }
            Ok(())
        })
    }

    fn get_all_repos(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<Repo>, GitMoverError>> + Send>> {
        let token = self.token.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!("https://{CODEBERG_URL}/api/v1/user/repos");
            let mut page: usize = 1;
            let limit = 100;
            let mut all_repos = Vec::new();
            loop {
                let request = client
                    .get(&url)
                    .header(AUTHORIZATION, format!("token {token}"))
                    .header(ACCEPT, "application/json")
                    .query(&[("page", &page.to_string()), ("limit", &limit.to_string())])
                    .send();

                let response = request.await?;
                if !response.status().is_success() {
                    let text = response.text().await?;
                    return Err(GitMoverError::new(GitMoverErrorKind::GetAllRepos)
                        .with_platform(PlatformType::Codeberg)
                        .with_text(&text));
                }
                let text = response.text().await?;
                let repos: Vec<CodebergRepo> = serde_json::from_str(&text)?;
                let mut page_repos: Vec<Repo> = repos.into_iter().map(|r| r.into()).collect();
                if page_repos.is_empty() {
                    break;
                }
                println!("Requested codeberg (page {}): {}", page, page_repos.len());
                all_repos.append(&mut page_repos);
                page += 1;
            }
            Ok(all_repos)
        })
    }

    fn delete_repo(
        &self,
        name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let name = name.to_string();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!(
                "https://{}/api/v1/repos/{}/{}",
                CODEBERG_URL,
                self.get_username(),
                encode(&name)
            );
            let request = client
                .delete(&url)
                .header(AUTHORIZATION, format!("token {token}"))
                .header(ACCEPT, "application/json")
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::RepoDeletion)
                    .with_platform(PlatformType::Codeberg)
                    .with_text(&text));
            }
            Ok(())
        })
    }
}
