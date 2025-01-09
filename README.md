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
  -s, --source <SOURCE>            The source platform (github, gitlab, codeberg) [aliases: from]
  -d, --destination <DESTINATION>  The destination platform (github, gitlab, codeberg) [aliases: to]
  -n, --no-forks                   Don't sync forked repositories
  -r, --resync                     Resync all repositories
  -c, --config <CONFIG>            Custom configuration file
  -v, --verbose...                 Verbose mode (-v, -vv, -vvv)
  -h, --help                       Print help
```

## License

- [MIT](LICENSE)
