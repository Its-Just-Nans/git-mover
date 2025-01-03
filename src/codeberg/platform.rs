use std::pin::Pin;

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{GitMoverError, GitMoverErrorKind},
    utils::{Platform, Repo},
};

use super::CODEBERG_URL;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CodebergPlatform {
    pub username: String,
    pub token: String,
}

impl CodebergPlatform {
    pub fn new(username: String, token: String) -> Self {
        Self { username, token }
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CodebergRepo {
    pub name: String,
    pub description: String,
    pub private: bool,
    #[serde(skip_serializing)]
    pub fork: bool,
}

impl From<CodebergRepo> for Repo {
    fn from(repo: CodebergRepo) -> Self {
        Repo {
            name: repo.name,
            description: repo.description,
            private: repo.private,
            fork: repo.fork,
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

    fn create_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let repo_name = repo.name.to_string();
        let description = repo.description.to_string();
        let private = repo.private;
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!("https://{}/api/v1/user/repos", CODEBERG_URL);
            let json_body = CodebergRepo {
                name: repo_name.to_string(),
                description: description.to_string(),
                private,
                fork: false, // not uset
            };
            let request = client
                .post(url)
                .header(AUTHORIZATION, format!("token {}", token))
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .json(&json_body)
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let get_repo = self.get_repo(repo_name.as_str()).await?;
                let json_body_as_repo = json_body.clone().into();
                if get_repo != json_body_as_repo {
                    eprintln!(
                        "Repository already exists with different configuration {:?} {:?}",
                        get_repo, json_body_as_repo
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
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!(
                "https://{}/api/v1/repos/{}/{}",
                CODEBERG_URL,
                self.get_username(),
                repo_name
            );
            let request = client
                .get(&url)
                .header(AUTHORIZATION, format!("token {}", token))
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::GetRepo).with_text(&text));
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
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!(
                "https://{}/api/v1/repos/{}/{}",
                CODEBERG_URL,
                self.get_username(),
                repo.name
            );
            let json_body = CodebergRepo {
                name: repo.name.to_string(),
                description: repo.description.to_string(),
                private: repo.private,
                fork: repo.fork, // not uset
            };
            let request = client
                .patch(url)
                .header(AUTHORIZATION, format!("token {}", token))
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .json(&json_body)
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::RepoEdition).with_text(&text));
            }
            Ok(())
        })
    }

    fn get_all_repos(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<Repo>, GitMoverError>> + Send>> {
        let token = self.token.clone();
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!("https://{}/api/v1/user/repos", CODEBERG_URL);
            let mut page: usize = 1;
            let limit = 100;
            let mut all_repos = Vec::new();
            loop {
                let request = client
                    .get(&url)
                    .header(AUTHORIZATION, format!("token {}", token))
                    .header(ACCEPT, "application/json")
                    .query(&[("page", &page.to_string()), ("limit", &limit.to_string())])
                    .send();

                let response = request.await?;
                if !response.status().is_success() {
                    let text = response.text().await?;
                    return Err(GitMoverError::new(GitMoverErrorKind::GetAllRepos).with_text(&text));
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
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!(
                "https://{}/api/v1/repos/{}/{}",
                CODEBERG_URL,
                self.get_username(),
                name
            );
            let request = client
                .delete(&url)
                .header(AUTHORIZATION, format!("token {}", token))
                .header(ACCEPT, "application/json")
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::RepoDeletion).with_text(&text));
            }
            Ok(())
        })
    }
}
