//! This module contains the Platform trait and PlatformType enum.

use crate::{
    errors::{GitMoverError, GitMoverErrorKind},
    utils::{check_ssh_access, Repo},
};
use serde::Deserialize;
use std::pin::Pin;

/// The Platform trait is used to interact with different git platforms.
pub trait Platform: Sync + Send {
    /// Check git access
    fn check_git_access(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>> {
        let url_ssh = format!("git@{}", self.get_remote_url());
        Box::pin(async move {
            let (stdin, stderr) = check_ssh_access(&url_ssh).await?;
            if stdin.contains(self.get_username()) || stderr.contains(self.get_username()) {
                Ok(())
            } else {
                Err(GitMoverError::new(GitMoverErrorKind::Platform)
                    .with_text(&format!(
                        "Cannot access to {url_ssh}: stdin={stdin} stderr={stderr}"
                    ))
                    .with_platform(self.get_type()))
            }
        })
    }

    /// Create a new repository on the platform.
    fn create_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>>;

    /// Get a repository from the platform.
    fn get_repo(
        &self,
        name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Repo, GitMoverError>> + Send + '_>>;

    /// Edit a repository on the platform.
    fn edit_repo(
        &self,
        repo: Repo,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>>;

    /// Get all repositories from the platform.
    fn get_all_repos(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<Repo>, GitMoverError>> + Send>>;

    /// Delete a repository from the platform.
    fn delete_repo(
        &self,
        name: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), GitMoverError>> + Send + '_>>;

    /// Get the username of the platform.
    fn get_username(&self) -> &str;

    /// Get the platform type.
    fn get_remote_url(&self) -> &str;

    /// get the type of the Platform
    fn get_type(&self) -> PlatformType;
}

/// The PlatformType enum is used to specify the platform type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum PlatformType {
    /// Gitlab platform
    Gitlab,

    /// Github platform
    Github,

    /// Codeberg platform
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
