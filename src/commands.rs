use crate::config::Config;
use reqwest::blocking::get;
use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::Path,
    process::Command,
};

const AUTOEXEC: &str = "autoexec.txt";
const EMU: &str = "fab-agon-emulator";
const EMU_ARGS: [&str; 3] = ["-f", "--sdcard", "./sdcard"];
const GIT: &str = "git";
const HELLO: &str = "PRINT \"Hello World!\"";
const MAIN: &str = "main.bas";
const SRC: &str = "src";
const TOML: &str = "Bargo.toml";

pub trait BargoCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>>;
    fn usage(action: Option<Action>) -> String;
}

#[derive(Debug)]
struct BargoError {
    message: String,
}

impl BargoError {
    fn new(message: &str) -> Self {
        Self {
            message: String::from(message),
        }
    }
}

impl std::fmt::Display for BargoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for BargoError {
    fn description(&self) -> &str {
        &self.message
    }
}

pub enum Action {
    DepAdd,
    DepRemove,
    Init,
    New,
    Unknown,
}

pub struct BuildCommand {
    config: Config,
}

impl BuildCommand {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: Config::read(TOML)?,
        })
    }

    fn fetch_dep(&self, url: &str, filename: &str) -> Result<(), Box<dyn Error>> {
        let response = get(url).map_err(|_| format!("Could not fetch {}", url))?;
        let body = response.text()?;
        let mut output =
            File::create(filename).map_err(|_| format!("Could not create {}", filename))?;
        write!(output, "{}", body).map_err(|_| format!("Could not write to {}", filename))?;
        println!("\tFetched {} to {}", url, filename);

        Ok(())
    }

    fn read_deps(&self, deps: &HashMap<String, String>) -> Result<Vec<String>, Box<dyn Error>> {
        let mut lines: Vec<String> = Vec::new();

        for (filename, url) in deps {
            if !url.is_empty() {
                self.fetch_dep(url, &format!("src/{}.bas", filename))?;
            }

            lines.push(String::from(":"));
            lines.push(format!("REM {}", "=".repeat(76)));
            lines.push(format!("REM IMPORT {}.BAS", filename.to_uppercase()));
            lines.push(format!("REM {}", "=".repeat(76)));
            lines.push(String::from(":"));

            let path = format!("{}/{}.bas", SRC, filename);
            let f = File::open(&path).map_err(|_| {
                format!(
                    "Could not open {}\nMake sure this dep exists in src/",
                    &path
                )
            })?;

            for line in BufReader::new(f).lines() {
                lines.push(line?)
            }
        }

        Ok(lines)
    }

    fn format_lines(&self, lines: &[String]) -> Vec<String> {
        let padding = (lines.len() * self.config.package.numbering)
            .to_string()
            .len();
        let mut labels = HashMap::new();

        if self.config.package.labels {
            for (number, line) in lines.iter().enumerate() {
                if line.starts_with("LABEL") {
                    let label = line.trim_start_matches("LABEL ");
                    labels.insert(
                        String::from(label),
                        (number + 2) * self.config.package.numbering,
                    );
                }
            }
        }

        lines
            .iter()
            .enumerate()
            .map(|(number, line)| {
                format!(
                    "{: >padding$} {}",
                    (number + 1) * self.config.package.numbering,
                    if line.to_uppercase().starts_with("LABEL") {
                        String::from(":")
                    } else if line.to_uppercase().starts_with("REM") && line.ends_with('=') {
                        line.chars()
                            .take(self.config.package.width - (padding + 3))
                            .collect()
                    } else {
                        let mut new_line = line.clone();
                        if self.config.package.labels {
                            for (label, number) in &labels {
                                new_line = new_line.replace(
                                    &format!("GOTO {}", label),
                                    &format!("GOTO {}", number),
                                );
                                new_line = new_line.replace(
                                    &format!("GOSUB {}", label),
                                    &format!("GOSUB {}", number),
                                );
                            }

                            new_line
                        } else {
                            new_line
                        }
                    }
                )
            })
            .collect()
    }
}

impl BargoCommand for BuildCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!(
            "\tBuilding {} v{}",
            self.config.package.name, self.config.package.version
        );
        let path = format!("{}/{}", SRC, MAIN);
        let f = File::open(&path).map_err(|_| format!("Could not open {}", &path))?;
        let mut lines: Vec<String> = Vec::new();
        let dependencies = self.config.dependencies.clone().unwrap_or_default();
        let dep_lines = self.read_deps(&dependencies)?;

        for line in BufReader::new(f).lines() {
            lines.push(line?);
        }

        for line in dep_lines {
            lines.push(line);
        }

        let formatted_lines = self.format_lines(&lines);
        let path = format!("{}.bas", self.config.package.name);
        let mut output = File::create(&path).map_err(|_| format!("Could not create {}", &path))?;

        for line in &formatted_lines {
            write!(
                output,
                "{}{}",
                line,
                if self.config.package.carriage_return {
                    "\r\n"
                } else {
                    "\n"
                }
            )
            .map_err(|_| format!("Could not write to {}", &path))?;
        }

        println!("\tFinished");

        Ok(())
    }

    fn usage(_: Option<Action>) -> String {
        String::from("\tbuild\tBuild the current package")
    }
}

pub struct DepCommand<'a> {
    action: Action,
    dependency: &'a str,
    config: RefCell<Config>,
}

impl<'a> DepCommand<'a> {
    pub fn new(dependency: &'a str, action: Action) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            action,
            config: RefCell::new(Config::read(TOML)?),
            dependency,
        })
    }
}

impl<'a> BargoCommand for DepCommand<'a> {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let mut config = self.config.borrow_mut();
        let mut dependencies = config.dependencies.clone().unwrap_or_default();
        match self.action {
            Action::DepAdd => {
                println!("\tAdding {} dependency", self.dependency);
                dependencies.insert(String::from(self.dependency), String::default());
            }
            Action::DepRemove => {
                println!("\tRemoving {} dependency", self.dependency);
                dependencies.remove(self.dependency);
            }
            _ => return Err(Box::new(BargoError::new("Unknown action"))),
        };
        config.dependencies = Some(dependencies);
        config.write(TOML)?;
        println!("\tFinished");
        Ok(())
    }

    fn usage(action: Option<Action>) -> String {
        match action {
            Some(Action::DepAdd) => String::from("\tadd\tAdd dependencies to this package"),
            Some(Action::DepRemove) => {
                String::from("\tremove\tRemove dependencies from this package")
            }
            _ => String::default(),
        }
    }
}

pub struct CleanCommand {
    config: Config,
}

impl CleanCommand {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: Config::read(TOML)?,
        })
    }
}

impl BargoCommand for CleanCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let path = format!("{}.bas", self.config.package.name);
        fs::remove_file(&path).map_err(|_| format!("Could not remove {}", &path))?;
        println!("\tRemoved {}.bas", self.config.package.name);

        Ok(())
    }

    fn usage(_: Option<Action>) -> String {
        String::from("\tclean\tRemove the generated file")
    }
}

pub struct EmuCommand {
    config: Config,
}

impl EmuCommand {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: Config::read(TOML)?,
        })
    }
}

impl BargoCommand for EmuCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let mut path = self.config.package.emu_path.clone();

        if !path.exists() {
            return Err(format!(
                "Could not find the emulator in {}\nSpecify the full path to the emulator's folder in {}",
                path.to_string_lossy(),
                TOML
            )
            .into());
        }

        path.push(format!("sdcard/{}.bas", self.config.package.name));
        fs::copy(format!("{}.bas", self.config.package.name), &path).map_err(|_| {
            "Could not copy source to emulator\nGo to project's root folder and/or build first"
        })?;

        path.pop();
        path.push(AUTOEXEC);
        let mut output = File::create(&path)
            .map_err(|_| format!("Could not create {}", &path.to_string_lossy()))?;
        write!(output, "bbcbasic /{}.bas\r\n", self.config.package.name)
            .map_err(|_| format!("Could not write to {}", &path.to_string_lossy()))?;

        path.pop();
        path.pop();
        let current_dir = path.clone();
        path.push(EMU);
        Command::new(path)
            .args(EMU_ARGS)
            .current_dir(current_dir)
            .output()
            .map_err(|_| "Could not run emulator")?;

        Ok(())
    }

    fn usage(_: Option<Action>) -> String {
        String::from("\temu\tRun the code inside an emulator")
    }
}

pub struct NewCommand<'a> {
    name: Option<&'a str>,
}

impl<'a> NewCommand<'a> {
    pub fn new(name: Option<&'a str>) -> Self {
        Self { name }
    }
}

impl<'a> BargoCommand for NewCommand<'a> {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let name = self.name.unwrap_or(".");
        let path = format!("{}/{}", name, SRC);

        if Path::new(&path).exists() {
            return Err("Package already exists".into());
        }

        fs::create_dir_all(&path).map_err(|_| format!("Could not create {}", &path))?;

        let mut config = Config::default();
        let path = format!("{}/{}", name, TOML);
        config.package.name = if name != "." {
            String::from(name)
        } else {
            let current_dir = env::current_dir().map_err(|_| "Could not get cwd")?;
            let file_name = current_dir.file_name().ok_or("Could not get cwd")?;
            let name = file_name.to_str().ok_or("Could not get cwd")?;
            String::from(name)
        };
        config.write(path)?;

        let path = format!("{}/{}/{}", name, SRC, MAIN);
        let mut output = File::create(&path).map_err(|_| format!("Could not create {}", &path))?;
        write!(output, "{}", HELLO).map_err(|_| format!("Could not write to {}", &path))?;

        println!("\tCreated `{}` package", config.package.name);

        Command::new(GIT)
            .arg("init")
            .current_dir(name)
            .output()
            .map_err(|_| "Could not run git to init repo")?;

        Ok(())
    }

    fn usage(action: Option<Action>) -> String {
        match action {
            Some(Action::New) => String::from("\tnew\tCreate a new Bargo package"),
            Some(Action::Init) => {
                String::from("\tinit\tCreate a new Bargo package in an existing directory")
            }
            _ => String::default(),
        }
    }
}
