//! Github Repo struct and conversion to Repo struct
use crate::utils::Repo;
use serde::{Deserialize, Serialize};

/// Github Repo
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct RepoGithub {
    /// Repository ID
    pub id: u64,

    /// Repository name
    pub name: String,

    /// Repository description
    pub description: Option<String>,

    /// Repository private status
    pub private: bool,

    /// Repository URL
    pub html_url: String,

    /// Repository fork status
    pub fork: bool,
}

impl From<RepoGithub> for Repo {
    fn from(repo: RepoGithub) -> Self {
        Repo {
            name: repo.name.clone(),
            path: repo.name,
            description: repo.description.unwrap_or_default(),
            private: repo.private,
            fork: repo.fork,
        }
    }
}
