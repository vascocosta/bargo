use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, read_to_string, File},
    io::{BufRead, BufReader, Write},
};

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
    dependencies: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            package: Package::default(),
            dependencies: HashMap::default(),
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
    let reader = BufReader::new(f);
    let lines = reader.lines();
    let numbered_lines: Result<Vec<String>, Box<dyn Error>> = lines
        .enumerate()
        .map(|(number, line)| match line {
            Ok(line) => Ok(format!(
                "{} {}",
                (number + 1) * config.package.numbering,
                line
            )),
            Err(err) => Err(err.into()), // If a line can't be read, convert error.
        })
        .collect();
    let mut output = File::create(format!("{}.bas", config.package.name))?;

    for line in &numbered_lines? {
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

    Ok(())
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
