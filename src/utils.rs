use std::collections::HashSet;
use std::{fmt::Debug, pin::Pin, sync::Arc};

use serde::Deserialize;
use tokio::join;

use crate::cli::GitMoverCli;
use crate::errors::GitMoverError;
use crate::sync::{delete_repos, sync_repos};
use crate::{codeberg::CodebergConfig, config::Config, github::GithubConfig, gitlab::GitlabConfig};

#[derive(Deserialize, Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct Repo {
    pub name: String,
    pub description: String,
    pub private: bool,
    pub fork: bool,
}

pub trait Platform: Debug + Sync + Send {
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

pub enum Direction {
    Source,
    Destination,
}

pub fn input_number() -> usize {
    loop {
        match input().parse::<usize>() {
            Ok(i) => return i,
            Err(_) => {
                println!("Invalid input");
            }
        }
    }
}

pub(crate) fn get_plateform(
    config: &mut Config,
    direction: Direction,
) -> (Arc<Box<dyn Platform>>, PlatformType) {
    let plateform_from_cli: Option<PlatformType> = match direction {
        Direction::Source => match &config.cli_args {
            Some(GitMoverCli {
                source: Some(source),
                ..
            }) => Some(source.clone().into()),
            _ => None,
        },
        Direction::Destination => match &config.cli_args {
            Some(GitMoverCli {
                destination: Some(destination),
                ..
            }) => Some(destination.clone().into()),
            _ => None,
        },
    };
    let chosen_platform = match plateform_from_cli {
        Some(platform) => platform,
        None => {
            println!(
                "Choose a platform {}",
                match direction {
                    Direction::Source => "for source",
                    Direction::Destination => "for destination",
                }
            );
            let platforms = [
                PlatformType::Github,
                PlatformType::Gitlab,
                PlatformType::Codeberg,
            ];
            for (i, platform) in platforms.iter().enumerate() {
                println!("{}: {:?}", i, platform);
            }
            let plateform = input_number();
            platforms[plateform]
        }
    };
    let correct: Box<dyn Platform> = match chosen_platform {
        PlatformType::Gitlab => Box::new(GitlabConfig::get_plateform(config)),
        PlatformType::Github => Box::new(GithubConfig::get_plateform(config)),
        PlatformType::Codeberg => Box::new(CodebergConfig::get_plateform(config)),
    };
    (Arc::new(correct), chosen_platform)
}

pub(crate) async fn main_sync(config: &mut Config) {
    let (source_plateform, type_source) = get_plateform(config, Direction::Source);
    println!("Chosen {} as source", source_plateform.get_remote_url());

    let (destination_platform, type_dest) = get_plateform(config, Direction::Destination);
    println!(
        "Chosen {} as destination",
        destination_platform.get_remote_url()
    );
    if type_source == type_dest {
        eprintln!("Source and destination can't be the same");
        return;
    }

    let (repos_source, repos_destination) = join!(
        source_plateform.get_all_repos(),
        destination_platform.get_all_repos()
    );

    let repos_source = match repos_source {
        Ok(repos) => repos,
        Err(e) => {
            eprintln!("Error getting repositories for source: {:?}", e);
            return;
        }
    };

    let repos_destination = match repos_destination {
        Ok(repos) => repos,
        Err(e) => {
            eprintln!("Error getting repositories for destination: {:?}", e);
            return;
        }
    };

    let repos_source_without_fork = repos_source
        .clone()
        .into_iter()
        .filter(|repo| !repo.fork)
        .collect::<Vec<_>>();
    let repos_source_forks = repos_source
        .clone()
        .into_iter()
        .filter(|repo| repo.fork)
        .collect::<Vec<_>>();
    println!("Number of repos in source: {}", repos_source.len());
    println!(
        "- Number of forked repos in source: {}",
        repos_source_forks.len()
    );
    println!(
        "- Number of (non-forked) repos in source: {}",
        repos_source_without_fork.len()
    );
    println!(
        "Number of repos in destination: {}",
        repos_destination.len()
    );
    let cloned_repos_source_without_fork = repos_source_without_fork.clone();
    let cloned_repos_destination = repos_destination.clone();
    let item_source_set: HashSet<_> = cloned_repos_source_without_fork.iter().collect();
    let item_destination_set: HashSet<_> = repos_destination.iter().collect();
    let missing_dest: Vec<Repo> = cloned_repos_destination
        .into_iter()
        .filter(|item| !item_source_set.contains(item))
        .collect();
    let difference: Vec<Repo> = cloned_repos_source_without_fork
        .into_iter()
        .filter(|item| !item_destination_set.contains(item))
        .collect();
    println!("Number of repos to sync: {}", difference.len());
    println!("Number of repos to delete: {}", missing_dest.len());
    if !difference.is_empty() && yes_no_input("Do you want to start syncing ? (y/n)") {
        match sync_repos(
            source_plateform.clone(),
            destination_platform.clone(),
            difference,
        )
        .await
        {
            Ok(_) => {
                println!("All repos synced");
            }
            Err(e) => {
                eprintln!("Error syncing repos: {:?}", e);
            }
        }
    }
    if let Some(GitMoverCli {
        no_forks: false, ..
    }) = &config.cli_args
    {
        if !repos_source_forks.is_empty()
            && yes_no_input(
                format!(
                    "Do you want to sync forks ({})? (y/n)",
                    repos_source_forks.len()
                )
                .as_str(),
            )
        {
            match sync_repos(
                source_plateform,
                destination_platform.clone(),
                repos_source_forks,
            )
            .await
            {
                Ok(_) => {
                    println!("All forks synced");
                }
                Err(e) => {
                    eprintln!("Error syncing forks: {:?}", e);
                }
            }
        }
    }
    if !missing_dest.is_empty()
        && yes_no_input("Do you want to delete the missing repos (manually)? (y/n)")
    {
        match delete_repos(destination_platform, missing_dest).await {
            Ok(_) => {
                println!("All repos deleted");
            }
            Err(e) => {
                eprintln!("Error deleting repos: {:?}", e);
            }
        }
    }
}

/// Get input from the user
pub(crate) fn input() -> String {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}

pub(crate) fn yes_no_input(msg: &str) -> bool {
    loop {
        println!("{}", msg);
        let input = input();
        match input.to_lowercase().as_str() {
            "yes" | "y" | "Y" => return true,
            "no" | "n" | "N" => return false,
            _ => println!("Invalid input"),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn compare_repo() {
        let repo1 = Repo {
            name: "test".to_string(),
            description: "test".to_string(),
            private: false,
            fork: false,
        };
        let repo2 = Repo {
            name: "test".to_string(),
            description: "test".to_string(),
            private: false,
            fork: false,
        };
        let repo3 = Repo {
            name: "test".to_string(),
            description: "test".to_string(),
            private: true,
            fork: false,
        };
        assert!(repo1 == repo2);
        assert!(repo1 != repo3);
        assert_eq!(repo1, repo2);
    }
}
