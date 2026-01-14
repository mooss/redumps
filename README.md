# Reddit dump processor

Process the Reddit dumps provided by https://academictorrents.com/details/1614740ac8c94505e4ecb9d88be8bed7b6afddd4.

This is also meant to be a Rust learning project.

## Features

For now does nothing more than counting the fields of a given uncompressed NLJSON subreddit file.

## Usage

```justfile
just run filename | tail # output can be quite long
```

## Requirements

 - A [Rust development environment](https://rust-lang.github.io/rustup/installation/index.html).
 - [`just`](https://github.com/casey/just)
