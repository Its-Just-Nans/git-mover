//! Gitlab Repo module
use crate::utils::Repo;
use serde::{Deserialize, Serialize};

/// Gitlab Repo
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GitlabRepo {
    /// Repo name
    pub name: String,

    /// Repo path
    pub path: String,

    /// Repo description
    pub description: Option<String>,

    /// Repo visibility
    pub visibility: String,

    /// Forked from project
    #[serde(skip_serializing)]
    pub forked_from_project: Option<ForkRepo>,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct ForkRepo {
    /// Forked from project id
    pub id: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GitlabRepoEdition {
    /// Repo name
    pub description: String,

    /// Repo visibility
    pub visibility: String,
}

impl From<GitlabRepo> for Repo {
    fn from(repo: GitlabRepo) -> Self {
        Repo {
            name: repo.name,
            path: repo.path,
            description: repo.description.unwrap_or_default(),
            private: repo.visibility == "private",
            fork: repo.forked_from_project.is_some(),
        }
    }
}
