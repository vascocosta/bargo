use crate::config::Config;
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
    fn usage() -> String;
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

    fn usage() -> String {
        String::from("\tclean\tRemove the generated file")
    }
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

    fn read_deps(&self, deps: &HashMap<String, String>) -> Result<Vec<String>, Box<dyn Error>> {
        let mut lines: Vec<String> = Vec::new();

        for filename in deps.keys() {
            lines.push(format!(":"));
            lines.push(format!("REM {}", "=".repeat(76)));
            lines.push(format!("REM IMPORT {}.BAS", filename.to_uppercase()));
            lines.push(format!("REM {}", "=".repeat(76)));
            lines.push(format!(":"));

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

        let padding = (lines.len() * self.config.package.numbering)
            .to_string()
            .len();
        let numbered_lines: Vec<String> = lines
            .into_iter()
            .enumerate()
            .map(|(number, line)| {
                format!(
                    "{: >padding$} {}",
                    (number + 1) * self.config.package.numbering,
                    line
                )
            })
            .collect();
        let path = format!("{}.bas", self.config.package.name);
        let mut output = File::create(&path).map_err(|_| format!("Could not create {}", &path))?;

        for line in &numbered_lines {
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

    fn usage() -> String {
        String::from("\tbuild\tBuild the current package")
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

    fn usage() -> String {
        format!(
            "{}\n{}",
            "\tinit\tCreate a new Bargo package in an existing directory",
            "\tnew\tCreate a new Bargo package"
        )
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
        write!(
            output,
            "load bbcbasic.bin\r\nrun . /{}.bas\r\n",
            self.config.package.name
        )
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

    fn usage() -> String {
        String::from("\temu\tRun the code inside an emulator")
    }
}

pub struct AddCommand<'a> {
    dependency: &'a str,
    config: RefCell<Config>,
}

impl<'a> AddCommand<'a> {
    pub fn new(dependency: &'a str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: RefCell::new(Config::read(TOML)?),
            dependency,
        })
    }
}

impl<'a> BargoCommand for AddCommand<'a> {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        let mut config = self.config.borrow_mut();
        let mut dependencies = config.dependencies.clone().unwrap_or_default();
        dependencies.insert(String::from(self.dependency), String::from("0.1.0"));
        config.dependencies = Some(dependencies);
        config.write(TOML)?;

        Ok(())
    }

    fn usage() -> String {
        String::from("\tadd\tAdd dependencies to this package")
    }
}
