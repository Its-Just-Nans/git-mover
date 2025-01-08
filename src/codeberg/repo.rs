//! Codeberg repository
use crate::utils::Repo;
use serde::{Deserialize, Serialize};

/// Codeberg repository
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CodebergRepo {
    /// Name of the repository
    pub name: String,

    /// Description of the repository
    pub description: String,

    /// Whether the repository is private
    pub private: bool,

    /// Whether the repository is a fork
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
