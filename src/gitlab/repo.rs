use crate::utils::Repo;
use serde::{Deserialize, Serialize};

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
