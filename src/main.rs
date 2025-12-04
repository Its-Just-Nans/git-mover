use git_mover::git_mover_main;
use std::process::exit;

#[tokio::main]
async fn main() {
    println!(concat!(
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION")
    ));
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
    match git_mover_main().await {
        Ok(_) => {
            exit(0);
        }
        Err(e) => {
            eprintln!("{e}");
            exit(1);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_main() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .format_target(false)
            .format_timestamp(None)
            .init();
        git_mover_main().await.unwrap();
    }
}
