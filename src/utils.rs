//! Utility functions
use std::collections::HashSet;
use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::join;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::cli::GitMoverCli;
use crate::errors::GitMoverError;
use crate::platform::{Platform, PlatformType};
use crate::sync::{delete_repos, sync_repos};
use crate::{
    codeberg::config::CodebergConfig, config::Config, github::config::GithubConfig,
    gitlab::config::GitlabConfig,
};

/// Repository information
#[derive(Deserialize, Serialize, Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct Repo {
    /// Name of the repository
    pub name: String,

    /// Description of the repository
    pub description: String,

    /// Whether the repository is private
    pub private: bool,

    /// Whether the repository is a fork
    pub fork: bool,
}

pub enum Direction {
    /// Source platform
    Source,
    /// Destination platform
    Destination,
}

/// Get a number from the user
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

/// check git access
pub(crate) async fn check_ssh_access(ssh_url: &str) -> Result<(String, String), GitMoverError> {
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
    config: &mut Config,
    direction: Direction,
) -> (Arc<Box<dyn Platform>>, PlatformType) {
    let plateform_from_cli: Option<PlatformType> = match direction {
        Direction::Source => match &config.cli_args {
            Some(GitMoverCli {
                source: Some(source),
                ..
            }) => Some(source.clone()),
            _ => None,
        },
        Direction::Destination => match &config.cli_args {
            Some(GitMoverCli {
                destination: Some(destination),
                ..
            }) => Some(destination.clone()),
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
                println!("{i}: {platform}");
            }
            let plateform = input_number();
            platforms[plateform].clone()
        }
    };
    let correct: Box<dyn Platform> = match chosen_platform {
        PlatformType::Gitlab => Box::new(GitlabConfig::get_plateform(config)),
        PlatformType::Github => Box::new(GithubConfig::get_plateform(config)),
        PlatformType::Codeberg => Box::new(CodebergConfig::get_plateform(config)),
    };
    (Arc::new(correct), chosen_platform)
}

/// Main function to sync repositories
pub async fn main_sync(config: &mut Config) {
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

    let (acc, acc2) = join!(
        source_plateform.check_git_access(),
        destination_platform.check_git_access()
    );
    match acc {
        Ok(_) => {
            println!("Checked access to {}", source_plateform.get_remote_url());
        }
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    }
    match acc2 {
        Ok(_) => {
            println!(
                "Checked access to {}",
                destination_platform.get_remote_url()
            );
        }
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    }
    let (repos_source, repos_destination) = join!(
        source_plateform.get_all_repos(),
        destination_platform.get_all_repos()
    );

    let repos_source = match repos_source {
        Ok(repos) => repos,
        Err(e) => {
            eprintln!("Error getting repositories for source: {e}");
            return;
        }
    };

    let repos_destination = match repos_destination {
        Ok(repos) => repos,
        Err(e) => {
            eprintln!("Error getting repositories for destination: {e}");
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
    let resync = matches!(&config.cli_args, Some(GitMoverCli { resync: true, .. }));
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
    if !difference.is_empty() && yes_no_input("Do you want to start syncing ? (y/n)") {
        match sync_repos(
            source_plateform.clone(),
            destination_platform.clone(),
            difference,
            config.debug,
        )
        .await
        {
            Ok(_) => {
                println!("All repos synced");
            }
            Err(e) => {
                eprintln!("Error syncing repos: {e}");
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
                config.debug,
            )
            .await
            {
                Ok(_) => {
                    println!("All forks synced");
                }
                Err(e) => {
                    eprintln!("Error syncing forks: {e}");
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
                eprintln!("Error deleting repos: {e}");
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

/// Get a yes/no input from the user
pub(crate) fn yes_no_input(msg: &str) -> bool {
    loop {
        println!("{msg}");
        let input = input();
        match input.to_lowercase().as_str() {
            "yes" | "y" | "Y" => return true,
            "no" | "n" | "N" => return false,
            _ => println!("Invalid input"),
        }
    }
}

/// Get password from the user
pub(crate) fn get_password() -> String {
    rpassword::read_password().expect("Error reading password")
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
