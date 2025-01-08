use serde::{Deserialize, Serialize};

use crate::utils::Repo;

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
