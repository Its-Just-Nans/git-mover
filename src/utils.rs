//! Utility functions
use std::collections::HashSet;
use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::join;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::errors::GitMoverError;
use crate::platform::{Platform, PlatformType};
use crate::sync::{delete_repos, sync_repos};
use crate::{
    codeberg::config::CodebergConfig, config::GitMoverConfig, github::config::GithubConfig,
    gitlab::config::GitlabConfig,
};

/// Repository information
#[derive(Deserialize, Serialize, Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct Repo {
    /// Name of the repository
    pub name: String,

    /// Path of the repository
    pub path: String,

    /// Description of the repository
    pub description: String,

    /// Whether the repository is private
    pub private: bool,

    /// Whether the repository is a fork
    pub fork: bool,
}

impl Repo {
    /// Show the full name of the repo, including its path (if different from the name)
    pub fn show_full_name(&self) -> String {
        let fmt_path = if self.name != self.path {
            format!(" at path '{}'", self.path)
        } else {
            "".into()
        };
        format!("{}{}", self.name, fmt_path)
    }
}

/// GIT direction
pub enum Direction {
    /// Source platform
    Source,
    /// Destination platform
    Destination,
}

/// Get a number from the user
pub fn input_number() -> Result<usize, GitMoverError> {
    loop {
        match input()?.parse::<usize>() {
            Ok(i) => return Ok(i),
            Err(_) => {
                println!("Invalid input");
            }
        }
    }
}

/// check git access
pub(crate) async fn check_ssh_access<S: AsRef<str>>(
    ssh_url: S,
) -> Result<(String, String), GitMoverError> {
    let ssh_url = ssh_url.as_ref();
    let result = timeout(Duration::from_secs(5), async {
        Command::new("ssh")
            .arg("-T")
            .arg(ssh_url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
    })
    .await;

    match result {
        Ok(Ok(output)) => {
            let stdout_str = str::from_utf8(&output.stdout)?.to_string();
            let stderr_str = str::from_utf8(&output.stderr)?.to_string();
            Ok((stdout_str, stderr_str))
        }
        Ok(Err(e)) => Err(e.into()),
        Err(e) => Err(e.into()),
    }
}

/// Get the platform to use
pub(crate) fn get_plateform(
    config: &mut GitMoverConfig,
    direction: Direction,
) -> Result<Box<dyn Platform>, GitMoverError> {
    let plateform_from_cli: Option<PlatformType> = match direction {
        Direction::Source => config.cli_args.source.clone(),
        Direction::Destination => config.cli_args.destination.clone(),
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
                println!("{i}: {platform}");
            }
            let plateform = loop {
                let plateform = input_number()?;
                if platforms.get(plateform).is_none() {
                    println!("Wrong number");
                    continue;
                } else {
                    break plateform;
                }
            };
            platforms[plateform].clone()
        }
    };
    let plateform: Box<dyn Platform> = match chosen_platform {
        PlatformType::Gitlab => Box::new(GitlabConfig::get_plateform(config)?),
        PlatformType::Github => Box::new(GithubConfig::get_plateform(config)?),
        PlatformType::Codeberg => Box::new(CodebergConfig::get_plateform(config)?),
    };
    Ok(plateform)
}

/// Main function to sync repositories
/// # Errors
/// Error if an error happens
pub async fn main_sync(config: GitMoverConfig) -> Result<(), GitMoverError> {
    let mut config = config;
    let source_platform = get_plateform(&mut config, Direction::Source)?;
    println!("Chosen {} as source", source_platform.get_remote_url());

    let destination_platform = get_plateform(&mut config, Direction::Destination)?;
    println!(
        "Chosen {} as destination",
        destination_platform.get_remote_url()
    );
    if source_platform.get_remote_url() == destination_platform.get_remote_url() {
        return Err("Source and destination can't be the same".into());
    }
    println!("Checking the git access for each plateform");
    let (acc, acc2) = join!(
        source_platform.check_git_access(),
        destination_platform.check_git_access()
    );
    match acc {
        Ok(_) => {
            println!("Checked access to {}", source_platform.get_remote_url());
        }
        Err(e) => return Err(e),
    }
    match acc2 {
        Ok(_) => {
            println!(
                "Checked access to {}",
                destination_platform.get_remote_url()
            );
        }
        Err(e) => return Err(e),
    }
    let source_platform = Arc::new(source_platform);
    let destination_platform = Arc::new(destination_platform);
    let (repos_source, repos_destination) = join!(
        source_platform.get_all_repos(),
        destination_platform.get_all_repos()
    );

    let repos_source = match repos_source {
        Ok(repos) => repos,
        Err(e) => {
            return Err(format!("Error getting repositories for source: {e}").into());
        }
    };

    let repos_destination = match repos_destination {
        Ok(repos) => repos,
        Err(e) => {
            return Err(format!("Error getting repositories for destination: {e}").into());
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
    let resync = config.cli_args.resync;
    let difference: Vec<Repo> = if resync {
        cloned_repos_source_without_fork
    } else {
        cloned_repos_source_without_fork
            .into_iter()
            .filter(|item| !item_destination_set.contains(item))
            .collect()
    };
    println!("Number of repos to sync: {}", difference.len());
    println!("Number of repos to delete: {}", missing_dest.len());
    if !difference.is_empty() && yes_no_input("Do you want to start syncing ? (y/n)")? {
        match sync_repos(
            &config,
            source_platform.clone(),
            destination_platform.clone(),
            difference,
        )
        .await
        {
            Ok(_) => {
                println!("All repos synced");
            }
            Err(e) => return Err(format!("Error syncing repos: {e}").into()),
        }
    }
    if config.cli_args.no_forks {
        println!("Not syncing forks");
    } else if repos_source_forks.is_empty() {
        println!("No forks found");
    } else if yes_no_input(format!(
        "Do you want to sync forks ({})? (y/n)",
        repos_source_forks.len()
    ))? {
        match sync_repos(
            &config,
            source_platform,
            destination_platform.clone(),
            repos_source_forks,
        )
        .await
        {
            Ok(_) => {
                println!("All forks synced");
            }
            Err(e) => {
                return Err(format!("Error syncing forks: {e}").into());
            }
        }
    }
    if config.cli_args.no_delete {
        println!("Not prompting for deletion");
    } else if missing_dest.is_empty() {
        println!("Nothing to delete");
    } else if yes_no_input(format!(
        "Do you want to delete the missing ({}) repos (manually)? (y/n)",
        missing_dest.len()
    ))? {
        match delete_repos(destination_platform, missing_dest).await {
            Ok(_) => {
                println!("All repos deleted");
            }
            Err(e) => {
                return Err(format!("Error deleting repos: {e}").into());
            }
        }
    }
    Ok(())
}

/// Get input from the user
pub(crate) fn input() -> Result<String, GitMoverError> {
    use std::io::{stdin, stdout, Write};
    let mut s = String::new();
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .map_err(|e| GitMoverError::new_with_source("Did not enter a correct string", e))?;
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Ok(s)
}

/// Get a yes/no input from the user
pub(crate) fn yes_no_input<S: AsRef<str>>(msg: S) -> Result<bool, GitMoverError> {
    let msg = msg.as_ref();
    loop {
        println!("{msg}");
        let input = input()?;
        match input.to_lowercase().as_str() {
            "yes" | "y" | "Y" | "YES" | "Yes " => return Ok(true),
            "no" | "n" | "N" | "NO" | "No" => return Ok(false),
            _ => println!("Invalid input"),
        }
    }
}

/// Get password from the user
pub(crate) fn get_password() -> Result<String, GitMoverError> {
    rpassword::read_password()
        .map_err(|e| GitMoverError::new_with_source("Error reading password", e))
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn compare_repo() {
        let repo1 = Repo {
            name: "test".to_string(),
            path: "test".to_string(),
            description: "test".to_string(),
            private: false,
            fork: false,
        };
        let repo2 = Repo {
            name: "test".to_string(),
            path: "test".to_string(),
            description: "test".to_string(),
            private: false,
            fork: false,
        };
        let repo3 = Repo {
            name: "test".to_string(),
            path: "test".to_string(),
            description: "test".to_string(),
            private: true,
            fork: false,
        };
        assert!(repo1 == repo2);
        assert!(repo1 != repo3);
        assert_eq!(repo1, repo2);
    }
}
