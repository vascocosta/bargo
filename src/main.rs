use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, read_to_string, File},
    io::{BufRead, BufReader, Write},
};

const HELLO: &str = "PRINT \"Hello World!\"";
const MAIN: &str = "main.bas";
const SRC: &str = "src";
const TOML: &str = "Bargo.toml";

enum Action {
    NEW,
    UNKNOWN,
}

#[derive(Deserialize, Serialize)]
struct Config {
    package: Package,
    dependencies: Option<HashMap<String, String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            package: Package::default(),
            dependencies: Some(HashMap::default()),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Package {
    name: String,
    carriage_return: bool,
    numbering: usize,
    version: String,
}

impl Default for Package {
    fn default() -> Self {
        Self {
            name: String::new(),
            carriage_return: true,
            numbering: 10,
            version: String::from("0.1.0"),
        }
    }
}

fn build() -> Result<(), Box<dyn Error>> {
    let config: Config = toml::from_str(&read_to_string(TOML)?)?;
    let f = File::open(format!("{}/{}", SRC, MAIN))?;
    let mut lines: Vec<String> = Vec::new();
    let dep_lines = read_deps(config.dependencies.unwrap_or_default())?;

    for line in BufReader::new(f).lines() {
        lines.push(line?);
    }

    for line in dep_lines {
        lines.push(line);
    }

    let padding = (lines.len() * config.package.numbering).to_string().len();
    let numbered_lines: Vec<String> = lines
        .into_iter()
        .enumerate()
        .map(|(number, line)| {
            format!(
                "{: >padding$} {}",
                (number + 1) * config.package.numbering,
                line
            )
        })
        .collect();
    let mut output = File::create(format!("{}.bas", config.package.name))?;

    for line in &numbered_lines {
        write!(
            output,
            "{}{}",
            line,
            if config.package.carriage_return {
                "\r\n"
            } else {
                "\n"
            }
        )?;
    }

    Ok(())
}

fn new(name: &str) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(format!("{}/{}", name, SRC))?;

    let mut config = Config::default();
    let mut output = File::create(format!("{}/{}", name, TOML))?;
    config.package.name = String::from(name);
    write!(output, "{}", toml::to_string(&config)?)?;

    let mut output = File::create(format!("{}/{}/{}", name, SRC, MAIN))?;
    write!(output, "{}", HELLO)?;

    Ok(())
}

fn read_deps(deps: HashMap<String, String>) -> Result<Vec<String>, Box<dyn Error>> {
    let mut lines: Vec<String> = Vec::new();

    for filename in deps.keys() {
        lines.push(format!(":"));
        lines.push(format!("REM {}", "=".repeat(76)));
        lines.push(format!("REM IMPORT {}.BAS", filename.to_uppercase()));
        lines.push(format!("REM {}", "=".repeat(76)));
        lines.push(format!(":"));

        let f = File::open(format!("{}/{}.bas", SRC, filename))?;

        for line in BufReader::new(f).lines() {
            lines.push(line?)
        }
    }

    Ok(lines)
}

fn show_usage(action: Option<Action>) {
    match action {
        Some(Action::NEW) => println!("Usage: new <name>"),
        Some(Action::UNKNOWN) => println!("Usage: <new|build>"),
        None => println!("Usage: <new|build>"),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.get(0) {
        Some(action) => match action.as_str() {
            "build" => build(),
            "new" => match args.get(1) {
                Some(name) => new(&name),
                None => Ok(show_usage(Some(Action::NEW))),
            },
            _ => Ok(show_usage(Some(Action::UNKNOWN))),
        },
        None => Ok(show_usage(None)),
    }
}
