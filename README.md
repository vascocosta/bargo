# Bargo

BASIC build system and package manager.

## Features

* Automatic line numbering
* Customisable line numbering
* Customisable newline chars
* Dependency management
* Project creation
* Project build

## Build

To build `bargo` you need the `Rust toolchain` as well as these `dependencies`:

* serde = "1.0.197"
* toml = "0.8.12"

Follow these steps to fetch and compile the source of `bargo` and its `dependencies`:

```
git clone https://github.com/vascocosta/bargo.git

cd bargo

cargo build --release
```

## Install

Simply copy `bargo/target/release/bargo` to a folder in your path (ex: `$HOME/bin`).

## Usage example

```
bargo new my_basic_project_name

cd my_basic_project_name

edit src/main.bas

bargo build
```

## Configuration example

Your BASIC project is configured by editing `my_basic_project_name/Bargo.toml` like this:

```
[package]
name = "reminder"
carriage_return = true
numbering = 10
version = "0.1.0"

[dependencies]
hello = "0.1.0"
goodbye = "0.1.0"
```