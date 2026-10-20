# MOVED TO https://github.com/Its-Just-Nans/n4n5-crate

# git-mover [![crates.io version](https://img.shields.io/crates/v/git-mover)](https://crates.io/crates/git-mover) ![crates.io downloads](https://img.shields.io/crates/d/git-mover)

Move git repositories to a new location

[![asciicast](https://asciinema.org/a/Lfge8LlwsR9A2dKYMNT3v9Nh4.svg)](https://asciinema.org/a/Lfge8LlwsR9A2dKYMNT3v9Nh4)

## Usage

```sh
cargo install git-mover
git-mover
```

## Arguments

```txt
Usage: git-mover [OPTIONS]

Options:
      --source <SOURCE>            The source platform (github, gitlab, codeberg) [aliases: --from]
      --destination <DESTINATION>  The destination platform (github, gitlab, codeberg) [aliases: --to]
      --no-forks                   Don't sync forked repositories
      --no-delete                  Don't delete repositories
      --resync                     Resync all repositories
      --config <CONFIG>            Custom configuration file path
      --show-config-path           Show the current config path and exit
      --manual                     Sync manually
  -v, --verbose...                 Verbose mode [aliases: -d]
  -h, --help                       Print help
```

## License

- [MIT](LICENSE)
