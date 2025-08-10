use git_mover::cli_main;

fn setup() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
}

#[tokio::main]
async fn main() {
    println!(concat!(
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION")
    ));
    setup();
    cli_main().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_main() {
        setup();
        cli_main().await;
    }
}
