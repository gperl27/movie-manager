# Movie manager

## About

This app aims to alleviate the pain of clutter and organization of managing videos files (mp4) locally in folders or external devices.
Movie Manager keeps track of your movie files and where they come from. The app provides the ability to search by filename and filter by origin folder.

## Requirements

- Rust (and Cargo)
- Elm
- Grunt (for frontend optimization)
- wget
- tar
- unzip


## Installation

`$ ./init.sh`

will download frontend assets and then compile the frontend files

`$ cargo run`

will install Rust dependencies and compile the source

Since this a rust project, any valid rust command hereafter should work

## Backlog

- Recursive folder checking (possibly use multi-threading)
- Improved error handling (front and backend)
- Use filenames to pull in file metadata from web api