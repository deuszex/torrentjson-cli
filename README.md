# Torrent-file to json-file cli converter

This project is simply put a terminal on command-line app, 
that takes a torrent file and turns it into a more readable json format.

## Install

This project is written in Rust. Also, I use cargo henceforth:

```
cargo build --release
cargo install
```

## Usage

Usage is either from the project folder with cargo:
```
cargo run "inputfile.torrent" "output.whatevs"
```

Or after installation from the command-line/terminal/console
```
torrentjson-cli "inputfile.torrent" "output.whatevs"
```

The input file should always be a .torrent file, 
the one you use to get open-source projects and such.

The output files extension is up to you, linux users are lucky 
for they don't need to write any extensions. On windows .txt should work.

## Contribute

PRs accepted.

## License

MIT © Lovas "Deusz" Zoltán Róbert
