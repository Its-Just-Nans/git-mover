use crate::errors::GitMoverError;
use crate::errors::GitMoverErrorKind;
use crate::utils::Platform;
use crate::utils::Repo;
use std::pin::Pin;

use reqwest::header::ACCEPT;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};

use super::GITLAB_URL;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GitlabPlatform {
    username: String,
    token: String,
}

impl GitlabPlatform {
    pub fn new(username: String, token: String) -> Self {
        Self { username, token }
    }
}

const SLASH: &str = "%2F";

impl Platform for GitlabPlatform {
    fn get_remote_url(&self) -> &str {
        GITLAB_URL
    }

    fn get_username(&self) -> &str {
        &self.username
    }

    fn create_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let repo = repo.clone();
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!("https://{}/api/v4/projects", GITLAB_URL);
            let visibility = if repo.private { "private" } else { "public" };
            let json_body = GitlabRepo {
                name: repo.name.to_string(),
                description: Some(repo.description.to_string()),
                visibility: visibility.to_string(),
                forked_from_project: None, // unused
            };
            let request = client
                .post(url)
                .header("PRIVATE-TOKEN", &token)
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .json(&json_body)
                .send();

            let response = request.await?;
            if !response.status().is_success() {
                let get_repo = self.get_repo(repo.name.as_str()).await?;
                let json_body_as_repo = json_body.into();
                if get_repo != json_body_as_repo {
                    return self.edit_repo(json_body_as_repo).await;
                } else {
                    return Ok(());
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
        let repo = repo.clone();
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!(
                "https://{}/api/v4/projects/{}{}{}",
                GITLAB_URL,
                self.get_username(),
                SLASH,
                repo.name
            );
            let json_body = GitlabRepoEdition {
                description: repo.description.to_string(),
                visibility: (if repo.private { "private" } else { "public" }).to_string(),
            };
            let request = client
                .put(url)
                .header("PRIVATE-TOKEN", &token)
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

    fn get_repo(
        &self,
        name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Repo, GitMoverError>> + Send>> {
        let token = self.token.clone();
        let name = name.to_string();
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!("https://{}/api/v4/projects", GITLAB_URL);
            let request = client
                .get(&url)
                .header("PRIVATE-TOKEN", &token)
                .query(&[("owned", "true"), ("search", name.as_str())])
                .send();

            let response = request.await?;

            if !response.status().is_success() {
                return Err(GitMoverError::new(GitMoverErrorKind::GetRepo));
            }
            let text = response.text().await?;
            let repos = serde_json::from_str::<Vec<GitlabRepo>>(&text)?;
            match repos.into_iter().next() {
                Some(repo) => Ok(repo.into()),
                None => Err(GitMoverError::new(GitMoverErrorKind::RepoNotFound).with_text(&text)),
            }
        })
    }

    fn get_all_repos(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<Repo>, GitMoverError>> + Send>> {
        let token = self.token.clone();
        Box::pin(async move {
            let client = reqwest::Client::new();
            let url = format!("https://{}/api/v4/projects", GITLAB_URL);
            let mut need_request = true;
            let mut page: usize = 1;
            let mut all_repos = vec![];
            while need_request {
                let request = client
                    .get(&url)
                    .header("PRIVATE-TOKEN", &token)
                    .header(ACCEPT, "application/json")
                    .query(&[
                        ("per_page", "100"),
                        ("page", &page.to_string()),
                        ("owned", "true"),
                    ])
                    .send();

                let response = request.await?;
                if !response.status().is_success() {
                    return Err(GitMoverError::new(GitMoverErrorKind::GetAllRepos));
                }
                let text = response.text().await?;
                let repos: Vec<GitlabRepo> = match serde_json::from_str(&text) {
                    Ok(repos) => repos,
                    Err(e) => return Err(e.into()),
                };
                let repos: Vec<Repo> = repos.into_iter().map(|r| r.into()).collect();
                if repos.is_empty() {
                    need_request = false;
                }
                println!("Requested gitlab (page {}): {}", page, repos.len());
                all_repos.extend(repos);
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
                "https://{}/api/v4/projects/{}{}{}",
                GITLAB_URL,
                self.get_username(),
                SLASH,
                name
            );
            let request = client
                .delete(&url)
                .header("PRIVATE-TOKEN", &token)
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

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GitlabRepo {
    pub name: String,
    pub description: Option<String>,
    pub visibility: String,
    #[serde(skip_serializing)]
    pub forked_from_project: Option<ForkRepo>,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ForkRepo {
    pub id: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GitlabRepoEdition {
    pub description: String,
    pub visibility: String,
}

impl From<GitlabRepo> for Repo {
    fn from(repo: GitlabRepo) -> Self {
        Repo {
            name: repo.name,
            description: repo.description.unwrap_or_default(),
            private: repo.visibility == "private",
            fork: repo.forked_from_project.is_some(),
        }
    }
}
