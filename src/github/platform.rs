use crate::{
    errors::{GitMoverError, GitMoverErrorKind},
    utils::{Platform, Repo},
};
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Client,
};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GithubPlatform {
    pub username: String,
    pub token: String,
}

impl GithubPlatform {
    pub fn new(username: String, token: String) -> Self {
        Self { username, token }
    }
}

impl Platform for GithubPlatform {
    fn get_remote_url(&self) -> &str {
        "github.com"
    }

    fn get_username(&self) -> &str {
        &self.username
    }

    fn create_repo(
        &self,
        _repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        unimplemented!("GitlabConfig::create_repo");
        Box::pin(async { Err(GitMoverError::new(GitMoverErrorKind::Unimplemented)) })
    }

    fn edit_repo(
        &self,
        _repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        unimplemented!("GitlabConfig::edit_repo");
        Box::pin(async { Err(GitMoverError::new(GitMoverErrorKind::Unimplemented)) })
    }

    fn get_repo(
        &self,
        _name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Repo, GitMoverError>> + Send>> {
        unimplemented!("GitlabConfig::get_repo");
        Box::pin(async { Err(GitMoverError::new(GitMoverErrorKind::Unimplemented)) })
    }

    fn get_all_repos(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<Repo>, GitMoverError>> + Send>> {
        let token = self.token.clone();
        Box::pin(async move {
            let client = Client::new();
            let url = "https://api.github.com/user/repos";
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
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .header(ACCEPT, "application/vnd.github+json")
                    .header(USER_AGENT, "reqwest")
                    .header("X-GitHub-Api-Version", "2022-11-28")
                    .send();
                let response = request.await?;
                if !response.status().is_success() {
                    let text = response.text().await?;
                    return Err(GitMoverError::new(GitMoverErrorKind::GetAllRepos).with_text(&text));
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
        _name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        unimplemented!("GitlabConfig::delete_repo");
        Box::pin(async { Err(GitMoverError::new(GitMoverErrorKind::Unimplemented)) })
    }
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct RepoGithub {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub private: bool,
    pub html_url: String,
    pub fork: bool,
}

impl From<RepoGithub> for Repo {
    fn from(repo: RepoGithub) -> Self {
        Repo {
            name: repo.name,
            description: repo.description.unwrap_or_default(),
            private: repo.private,
            fork: repo.fork,
        }
    }
}
