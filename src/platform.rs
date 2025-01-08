use std::pin::Pin;

use serde::Deserialize;

use crate::{errors::GitMoverError, utils::Repo};





pub trait Platform: Sync + Send {
    fn create_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>>;

    fn get_repo(
        &self,
        name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Repo, GitMoverError>> + Send + '_>>;

    fn edit_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>>;

    fn get_all_repos(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<Repo>, GitMoverError>> + Send>>;

    fn delete_repo(
        &self,
        name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>>;

    fn get_username(&self) -> &str;
    fn get_remote_url(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum PlatformType {
    Gitlab,
    Github,
    Codeberg,
}

impl std::fmt::Display for PlatformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformType::Gitlab => write!(f, "gitlab"),
            PlatformType::Github => write!(f, "github"),
            PlatformType::Codeberg => write!(f, "codeberg"),
        }
    }
}

impl From<String> for PlatformType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "gitlab" => PlatformType::Gitlab,
            "github" => PlatformType::Github,
            "codeberg" => PlatformType::Codeberg,
            _ => panic!("Invalid platform"),
        }
    }
}
