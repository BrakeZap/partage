# **Partage**

## Command Line File Sharing
A simple CLI tool to easily share files between many users. The server and the command line client are both built entirely in [Rust](https://www.rust-lang.org/).

## Install:
Currently the only way to install and use the client is to install it from source. You will need Rust in order to build and run the executable. 

> [!Note]
> You can install Rust using rustup from here: [rustup](https://rustup.rs/).

Once Rust is installed, clone the repository, cd into the directory and then run `cargo build --release`. Now you can run the executable. 

## Usage:

Instructions on how to use the tool can be found using `partage help`. Remember that you must provide a valid http address to a running partage server in order to upload and download files. 
