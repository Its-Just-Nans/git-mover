//! Sync repositories from one platform to another
use git2::Cred;
use rand::{distr::Alphanumeric, rng, Rng};
use std::{fs::remove_dir_all, path::PathBuf, sync::Arc};
use tokio::task::JoinSet;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::errors::GitMoverError;
use crate::platform::Platform;
use crate::utils::{yes_no_input, Repo};
use crate::Config;

/// Sync repositories from one platform to another
pub(crate) async fn sync_repos(
    config: &Config,
    source_platform: Arc<Box<dyn Platform>>,
    destination_platform: Arc<Box<dyn Platform>>,
    repos: Vec<Repo>,
) -> Result<(), GitMoverError> {
    let rand_string: String = rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let temp_folder = std::env::temp_dir().join(format!("tmp-{rand_string}"));
    std::fs::create_dir(&temp_folder)?;

    let mut set = JoinSet::new();
    let verbose = config.debug;

    let mut private_repos = vec![];
    let m = Arc::new(MultiProgress::new());
    let total = repos.len();
    for (idx, one_repo) in repos.into_iter().enumerate() {
        if one_repo.private {
            private_repos.push(one_repo);
            continue;
        }
        let source_ref = source_platform.clone();
        let destination_ref = destination_platform.clone();
        let temp_dir_ref = temp_folder.clone();
        let repo_name = one_repo.name.clone();
        let sync_repo = async move |repo_name, one_repo, pb| match sync_one_repo(
            source_ref,
            destination_ref,
            one_repo,
            temp_dir_ref,
            (verbose, &pb),
        )
        .await
        {
            Ok(_) => {
                pb.finish_with_message(format!("{repo_name}: Successfully synced"));
            }
            Err(e) => {
                pb.finish_with_message(format!("{repo_name}: Error syncing {e}"));
            }
        };
        let create_pb = |m: &Arc<MultiProgress>, idx, total| -> ProgressBar {
            let pb = m.add(ProgressBar::new(10));
            if let Some(style) = get_style() {
                pb.set_style(style);
            }
            pb.set_prefix(format!("[{}/{}]", idx + 1, total));
            pb
        };
        if config.cli_args.manual {
            let question = format!("Should sync repo {} (y/n)", &repo_name);
            let should_sync = yes_no_input(&question);
            let pb = create_pb(&m, idx, total);
            match should_sync {
                true => {
                    sync_repo(repo_name, one_repo, pb).await;
                }
                false => {
                    pb.finish_with_message(format!("{repo_name}: Not synced"));
                }
            };
        } else {
            let pb = create_pb(&m, idx, total);
            set.spawn(async move { sync_repo(repo_name, one_repo, pb).await });
        }
    }
    let temp_folder_priv = temp_folder.clone();
    let progress = m.clone();
    let sync_private = async move || match sync_private_repos(
        source_platform,
        destination_platform,
        private_repos,
        temp_folder_priv,
        verbose,
        progress,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error syncing private repos: {e}");
        }
    };
    if config.cli_args.manual {
        sync_private().await;
    } else {
        set.spawn(async move { sync_private().await });
        set.join_all().await;
    }

    println!("Cleaning up {}", temp_folder.display());
    remove_dir_all(temp_folder)?;
    Ok(())
}

/// Sync private repositories from one platform to another
async fn sync_private_repos(
    source_platform: Arc<Box<dyn Platform>>,
    destination_platform: Arc<Box<dyn Platform>>,
    private_repos: Vec<Repo>,
    temp_folder: PathBuf,
    verbose: u8,
    progress: Arc<MultiProgress>,
) -> Result<(), GitMoverError> {
    let total = private_repos.len();
    for (idx, one_repo) in private_repos.into_iter().enumerate() {
        let question = format!(
            "Should sync private repo {} (y/n)",
            one_repo.show_full_name()
        );
        match yes_no_input(&question) {
            true => {
                let repo_name = one_repo.name.clone();
                let source_ref = source_platform.clone();
                let destination_ref = destination_platform.clone();
                let pb = progress.add(ProgressBar::new(10));
                if let Some(style) = get_style() {
                    pb.set_style(style);
                }
                pb.set_prefix(format!("[{}/{}]", idx + 1, total));
                match sync_one_repo(
                    source_ref,
                    destination_ref,
                    one_repo,
                    temp_folder.clone(),
                    (verbose, &pb),
                )
                .await
                {
                    Ok(_) => {
                        pb.finish_with_message(format!("{repo_name}: Successfully synced"));
                    }
                    Err(e) => {
                        pb.finish_with_message(format!("{repo_name}: Error syncing {e}"));
                    }
                }
            }
            false => {
                println!("Skipping {}", one_repo.show_full_name());
            }
        }
    }
    Ok(())
}

/// get ProgressStyle
fn get_style() -> Option<ProgressStyle> {
    match ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}") {
        Ok(s) => Some(s.tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")),
        Err(_) => None,
    }
}

/// Sync one repository from one platform to another
async fn sync_one_repo(
    source_platform: Arc<Box<dyn Platform>>,
    destination_platform: Arc<Box<dyn Platform>>,
    repo: Repo,
    temp_folder: PathBuf,
    verbosity: (u8, &ProgressBar),
) -> Result<(), GitMoverError> {
    let repo_cloned = repo.clone();
    let repo_name = repo.name.clone();
    let (_verbose, pb) = verbosity;
    let loog = |log_line: &str| {
        pb.set_message(format!("{repo_name}: {log_line}"));
        pb.inc(1);
    };
    loog("Start syncing");
    let tmp_repo_path = temp_folder.join(format!("{repo_name}.git"));

    loog("Creating repo to destination...");
    destination_platform.create_repo(repo_cloned).await?;
    loog("Creating repo to destination done");
    let source_platform = source_platform.as_ref();
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed| {
        let username = username_from_url.unwrap_or("git");
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

    loog(&format!(
        "Cloning from '{}' to '{}'...",
        url,
        tmp_repo_path.display(),
    ));
    let repo = builder.clone(&url, &tmp_repo_path)?;
    loog(&format!(
        "Cloning from '{}' to '{}' done",
        url,
        tmp_repo_path.display(),
    ));
    let next_remote = format!(
        "git@{}:{}/{}.git",
        destination_platform.get_remote_url(),
        destination_platform.get_username(),
        &repo_name
    );
    let new_remote_name = "new_origin";
    loog(&format!(
        "Adding remote {} to {}",
        new_remote_name, next_remote
    ));
    let mut remote = repo.remote(new_remote_name, &next_remote)?;

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed| {
        let username = username_from_url.unwrap_or("git");
        Ok(Cred::ssh_key_from_agent(username).expect("Could not get ssh key from ssh agent"))
    });
    loog(&format!("Connecting in push mode to {}", next_remote));
    remote.connect_auth(git2::Direction::Push, Some(callbacks), None)?;

    let refs = repo.references()?;
    for reference in refs {
        let reference = reference?;
        let ref_name = match reference.name() {
            Some(name) => name,
            None => continue,
        };
        loog(&format!("Pushing '{ref_name}'..."));
        let ref_remote = format!("+{ref_name}:{ref_name}");
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |_url, username_from_url, _allowed| {
            let username = username_from_url.unwrap_or("git");
            Ok(Cred::ssh_key_from_agent(username).expect("Could not get ssh key from ssh agent"))
        });
        let mut opts = git2::PushOptions::new();
        opts.remote_callbacks(callbacks);
        remote.push(&[&ref_remote], Some(&mut opts))?;
        loog(&format!("Pushing '{ref_name}' done"));
    }
    remove_dir_all(tmp_repo_path)?;
    Ok(())
}

/// Delete repositories from a platform
pub(crate) async fn delete_repos(
    destination_platform: Arc<Box<dyn Platform>>,
    repos: Vec<Repo>,
) -> Result<(), GitMoverError> {
    for (idx, one_repo) in repos.iter().enumerate() {
        let question = format!(
            "Should delete repo '{}' ({}/{}) (y/n)",
            one_repo.show_full_name(),
            idx,
            repos.len()
        );
        let should_delete = yes_no_input(&question);
        if should_delete {
            match destination_platform.delete_repo(&one_repo.path).await {
                Ok(_) => {
                    println!("Deleted {}", one_repo.show_full_name());
                }
                Err(e) => {
                    println!("Error: {e}");
                }
            }
        } else {
            println!("Skipping {}", one_repo.show_full_name());
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
            println!("Authenticating for URL: {_url}");
            println!("Username from URL: {username_from_url:?}");
            println!("Allowed types: {_allowed:?}");

            let username: &str = username_from_url.unwrap_or("git");
            Ok(Cred::ssh_key_from_agent(username).expect("Could not get ssh key from ssh agent"))
        });
        let mut builder = git2::build::RepoBuilder::new();
        builder.bare(true);
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        builder.fetch_options(fetch_opts);

        let url = "git@github.com:Its-Just-Nans/git-mover.git";
        println!("Cloning {url}");
        let _repo = match builder.clone(url, &PathBuf::from("git-mover")) {
            Ok(repo) => repo,
            Err(e) => {
                eprintln!("Error: {e}");
                return;
            }
        };
    }
}
