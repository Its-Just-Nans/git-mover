use serde::{Deserialize, Serialize};

use crate::utils::Repo;

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
