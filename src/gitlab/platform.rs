//! Gitlab platform implementation
use crate::errors::GitMoverError;
use crate::errors::GitMoverErrorKind;
use crate::platform::Platform;
use crate::platform::PlatformType;
use crate::utils::Repo;
use reqwest::header::ACCEPT;
use reqwest::header::CONTENT_TYPE;
use std::pin::Pin;
use urlencoding::encode;

use super::repo::GitlabRepo;
use super::repo::GitlabRepoEdition;
use super::GITLAB_URL;

/// Gitlab platform
#[derive(Default, Debug, Clone)]
pub struct GitlabPlatform {
    /// Gitlab username
    username: String,

    /// Gitlab token
    token: String,

    /// Reqwest client
    client: reqwest::Client,
}

impl GitlabPlatform {
    /// Create a new Gitlab platform
    pub fn new(username: String, token: String) -> Self {
        Self {
            username,
            token,
            client: reqwest::Client::new(),
        }
    }
}

impl Platform for GitlabPlatform {
    fn get_remote_url(&self) -> &str {
        GITLAB_URL
    }

    fn get_username(&self) -> &str {
        &self.username
    }

    fn get_type(&self) -> PlatformType {
        PlatformType::Gitlab
    }

    fn create_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let token = self.token.clone();
        let repo = repo.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!("https://{GITLAB_URL}/api/v4/projects");
            let visibility = if repo.private { "private" } else { "public" };
            let json_body = GitlabRepo {
                name: repo.name.to_string(),
                path: repo.path.to_string(),
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
                let text = response.text().await?;
                let get_repo = match self.get_repo(repo.path.as_str()).await {
                    Ok(repo) => repo,
                    Err(e) => {
                        let text_error = format!("{} - {}", &text, e);
                        return Err(GitMoverError::new(GitMoverErrorKind::RepoCreation)
                            .with_platform(PlatformType::Gitlab)
                            .with_text(&text_error));
                    }
                };
                let json_body_as_repo = json_body.into();
                if get_repo != json_body_as_repo {
                    return self.edit_repo(json_body_as_repo).await;
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
        let client = self.client.clone();
        Box::pin(async move {
            let repo_url = format!("{}/{}", self.get_username(), repo.path);
            let url = format!(
                "https://{}/api/v4/projects/{}",
                GITLAB_URL,
                encode(&repo_url),
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
                return Err(GitMoverError::new(GitMoverErrorKind::RepoEdition)
                    .with_platform(PlatformType::Gitlab)
                    .with_text(&text));
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
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!("https://{GITLAB_URL}/api/v4/projects");
            let request = client
                .get(&url)
                .header("PRIVATE-TOKEN", &token)
                .query(&[("owned", "true"), ("search", name.as_str())])
                .send();
            let response = request.await?;
            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::GetRepo)
                    .with_platform(PlatformType::Gitlab)
                    .with_text(&text));
            }
            let text = response.text().await?;
            let repos = serde_json::from_str::<Vec<GitlabRepo>>(&text)?;
            match repos.into_iter().next() {
                Some(repo) => Ok(repo.into()),
                None => Err(GitMoverError::new(GitMoverErrorKind::RepoNotFound)
                    .with_platform(PlatformType::Gitlab)
                    .with_text(&text)),
            }
        })
    }

    fn get_all_repos(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<Repo>, GitMoverError>> + Send>> {
        let token = self.token.clone();
        let client = self.client.clone();
        Box::pin(async move {
            let url = format!("https://{GITLAB_URL}/api/v4/projects");
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
                    let text = response.text().await?;
                    return Err(GitMoverError::new(GitMoverErrorKind::GetAllRepos)
                        .with_platform(PlatformType::Gitlab)
                        .with_text(&text));
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
        let client = self.client.clone();
        Box::pin(async move {
            let repo_url = format!("{}/{}", self.get_username(), name);
            let url = format!(
                "https://{}/api/v4/projects/{}",
                GITLAB_URL,
                encode(&repo_url),
            );
            let request = client
                .delete(&url)
                .header("PRIVATE-TOKEN", &token)
                .header(ACCEPT, "application/json")
                .send();

            let response = request.await?;

            if !response.status().is_success() {
                let text = response.text().await?;
                return Err(GitMoverError::new(GitMoverErrorKind::RepoDeletion)
                    .with_platform(PlatformType::Gitlab)
                    .with_text(&text));
            }
            Ok(())
        })
    }
}
