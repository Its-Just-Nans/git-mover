use git2::Cred;
use rand::thread_rng;
use rand::{distributions::Alphanumeric, Rng};
use std::{fs::remove_dir_all, path::PathBuf, sync::Arc};
use tokio::task::JoinSet;

use crate::errors::GitMoverError;
use crate::platform::Platform;
use crate::utils::{yes_no_input, Repo};

pub(crate) async fn sync_repos(
    source_platform: Arc<Box<dyn Platform>>,
    destination_platform: Arc<Box<dyn Platform>>,
    repos: Vec<Repo>,
) -> Result<(), GitMoverError> {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let temp_folder = std::env::temp_dir().join(format!("tmp-{}", rand_string));
    std::fs::create_dir(&temp_folder)?;

    let mut set = JoinSet::new();

    let mut private_repos = vec![];
    for one_repo in repos {
        if one_repo.private {
            private_repos.push(one_repo);
            continue;
        }
        let source_ref = source_platform.clone();
        let destination_ref = destination_platform.clone();
        let temp_dir_ref = temp_folder.clone();
        set.spawn(async move {
            let repo_name = one_repo.name.clone();
            match sync_one_repo(source_ref, destination_ref, one_repo, temp_dir_ref).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error syncing {}: {:?}", repo_name, e);
                }
            }
        });
    }
    let temp_folder_priv = temp_folder.clone();
    set.spawn(async move {
        match sync_private_repos(
            source_platform,
            destination_platform,
            private_repos,
            temp_folder_priv,
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error syncing private repos: {:?}", e);
            }
        }
    });

    set.join_all().await;
    println!("Cleaning up {}", temp_folder.display());
    remove_dir_all(temp_folder)?;
    Ok(())
}

async fn sync_private_repos(
    source_platform: Arc<Box<dyn Platform>>,
    destination_platform: Arc<Box<dyn Platform>>,
    private_repos: Vec<Repo>,
    temp_folder: PathBuf,
) -> Result<(), GitMoverError> {
    for one_repo in private_repos {
        let question = format!("Should sync private repo {} (y/n)", one_repo.name);
        match yes_no_input(&question) {
            true => {
                let source_ref = source_platform.clone();
                let destination_ref = destination_platform.clone();
                sync_one_repo(source_ref, destination_ref, one_repo, temp_folder.clone()).await?;
            }
            false => {
                println!("Skipping {}", one_repo.name);
            }
        }
    }
    Ok(())
}

async fn sync_one_repo(
    source_platform: Arc<Box<dyn Platform>>,
    destination_platform: Arc<Box<dyn Platform>>,
    repo: Repo,
    temp_folder: PathBuf,
) -> Result<String, GitMoverError> {
    let repo_cloned = repo.clone();
    let repo_name = repo.name.clone();
    let log_text = format!("(background) Start syncing '{}'", repo_name);
    let tmp_repo_path = temp_folder.join(format!("{}.git", repo_name));

    destination_platform.create_repo(repo_cloned).await?;
    let source_platform = source_platform.as_ref();
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed| {
        let username: &str = username_from_url.unwrap_or("git");
        Ok(Cred::ssh_key_from_agent(username).expect("Could not get ssh key from ssh agent"))
    });

    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);
    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    builder.fetch_options(fetch_opts);

    let url = format!(
        "git@{}:{}/{}.git",
        source_platform.get_remote_url(),
        source_platform.get_username(),
        &repo_name
    );
    let log_text = format!(
        "{}\n(background) Cloning '{}' at '{}'",
        log_text,
        url,
        tmp_repo_path.display()
    );
    let log_text = &format!("{}\n(background) Cloning from '{}'", log_text, url);
    let repo = builder.clone(&url, &tmp_repo_path)?;

    let next_remote = format!(
        "git@{}:{}/{}.git",
        destination_platform.get_remote_url(),
        destination_platform.get_username(),
        &repo_name
    );
    let mut remote = repo.remote("gitlab", &next_remote)?;

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed| {
        let username = username_from_url.unwrap_or("git");
        Ok(Cred::ssh_key_from_agent(username).expect("Could not get ssh key from ssh agent"))
    });
    remote.connect_auth(git2::Direction::Push, Some(callbacks), None)?;

    let refs = repo.references()?;
    let mut log_text = log_text.clone();
    for reference in refs {
        let reference = reference?;
        let ref_name = match reference.name() {
            Some(name) => name,
            None => continue,
        };
        log_text.push_str(&format!("\n(background) Pushing '{}'", ref_name));
        let ref_remote = format!("+{}:{}", ref_name, ref_name);
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |_url, username_from_url, _allowed| {
            let username = username_from_url.unwrap_or("git");
            Ok(Cred::ssh_key_from_agent(username).expect("Could not get ssh key from ssh agent"))
        });
        let mut opts = git2::PushOptions::new();
        opts.remote_callbacks(callbacks);
        match remote.push(&[ref_remote.clone()], Some(&mut opts)) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error for {} pushing {}: {}", repo_name, ref_remote, e);
            }
        }
    }
    remove_dir_all(tmp_repo_path)?;
    let log_text = format!(
        "{}\n(background) Finished syncing '{}'",
        log_text, repo_name
    );
    Ok(log_text)
}

pub(crate) async fn delete_repos(
    destination_platform: Arc<Box<dyn Platform>>,
    repos: Vec<Repo>,
) -> Result<(), GitMoverError> {
    for one_repo in repos {
        let question = format!("Should delete repo {} (y/n)", one_repo.name);
        match yes_no_input(&question) {
            true => {
                destination_platform.delete_repo(&one_repo.name).await?;
            }
            false => {
                println!("Skipping {}", one_repo.name);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    #[ignore] // This test is ignored because it requires a valid ssh key
    fn test_git_connection() {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |_url, username_from_url, _allowed| {
            println!("Authenticating for URL: {}", _url);
            println!("Username from URL: {:?}", username_from_url);
            println!("Allowed types: {:?}", _allowed);

            let username: &str = username_from_url.unwrap_or("git");
            Ok(Cred::ssh_key_from_agent(username).expect("Could not get ssh key from ssh agent"))
        });
        let mut builder = git2::build::RepoBuilder::new();
        builder.bare(true);
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        builder.fetch_options(fetch_opts);

        let url = "git@github.com:Its-Just-Nans/git-mover.git";
        println!("Cloning {}", url);
        let _repo = match builder.clone(url, &PathBuf::from("git-mover")) {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return;
            }
        };
    }
}
