use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, read_to_string, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
};

const AUTOEXEC: &str = "autoexec.txt";
const EMU: &str = "fab-agon-emulator";
const EMU_ARGS: &str = "-f";
const GIT: &str = "git";
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
    emu_path: PathBuf,
    version: String,
}

impl Default for Package {
    fn default() -> Self {
        let home = env::var("HOME").unwrap_or(String::from("./"));
        let mut emu_path = PathBuf::new();
        emu_path.push(&home);
        emu_path.push(EMU);

        Self {
            name: String::new(),
            carriage_return: true,
            numbering: 10,
            emu_path,
            version: String::from("0.1.0"),
        }
    }
}

fn build() -> Result<(), Box<dyn Error>> {
    let config: Config =
        toml::from_str(&read_to_string(TOML).map_err(|_| format!("Could not open {}", TOML))?)
            .map_err(|_| format!("Syntax error in {}", TOML))?;
    println!(
        "\tBuilding {} v{}",
        &config.package.name, &config.package.version
    );
    let path = format!("{}/{}", SRC, MAIN);
    let f = File::open(&path).map_err(|_| format!("Could not open {}", &path))?;
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
    let path = format!("{}.bas", config.package.name);
    let mut output = File::create(&path).map_err(|_| format!("Could not create {}", &path))?;

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
        )
        .map_err(|_| format!("Could not write to {}", &path))?;
    }

    println!("\tFinished");

    Ok(())
}

fn clean() -> Result<(), Box<dyn Error>> {
    let config: Config =
        toml::from_str(&read_to_string(TOML).map_err(|_| format!("Could not open {}", TOML))?)
            .map_err(|_| format!("Syntax error in {}", TOML))?;
    let path = format!("{}.bas", &config.package.name);
    fs::remove_file(&path).map_err(|_| format!("Could not remove {}", &path))?;
    println!("\tRemoved {}.bas", &config.package.name);

    Ok(())
}

fn emulator() -> Result<(), Box<dyn Error>> {
    let config: Config =
        toml::from_str(&read_to_string(TOML).map_err(|_| format!("Could not open {}", TOML))?)
            .map_err(|_| format!("Syntax error in {}", TOML))?;

    // Check if we can find the emulator folder.
    let mut path = config.package.emu_path;

    if !path.exists() {
        return Err(format!("Could not find emulator in {}", path.to_string_lossy()).into());
    }

    // Copy source code to emulator folder.
    path.push(format!("sdcard/{}.bas", &config.package.name));
    fs::copy(format!("{}.bas", &config.package.name), &path)
        .map_err(|_| "Could not copy source to emulator")?;

    // Generate autoexec.txt on the emulator.
    path.pop();
    path.push(AUTOEXEC);
    let mut output =
        File::create(&path).map_err(|_| format!("Could not create {}", &path.to_string_lossy()))?;
    write!(
        output,
        "load bbcbasic.bin\r\nrun . /{}.bas\r\n",
        &config.package.name
    )
    .map_err(|_| format!("Could not write to {}", &path.to_string_lossy()))?;

    // Execute the emulator with the source code.
    path.pop();
    path.pop();
    let current_dir = path.clone();
    path.push(EMU);
    Command::new(path)
        .arg(EMU_ARGS)
        .current_dir(current_dir)
        .output()
        .map_err(|_| "Could not run emulator")?;

    Ok(())
}

fn new(name: Option<&str>) -> Result<(), Box<dyn Error>> {
    let path = format!("{}/{}", name.unwrap_or("."), SRC);

    if Path::new(&path).exists() {
        return Err("Package already exists".into());
    }

    fs::create_dir_all(&path).map_err(|_| format!("Could not create {}", &path))?;

    let mut config = Config::default();
    let path = format!("{}/{}", name.unwrap_or("."), TOML);
    let mut output = File::create(&path).map_err(|_| format!("Could not create {}", &path))?;
    config.package.name = if let Some(name) = name {
        String::from(name)
    } else {
        let current_dir = env::current_dir().map_err(|_| "Could not get cwd")?;
        let file_name = current_dir.file_name().ok_or("Could not get cwd")?;
        let name = file_name.to_str().ok_or("Could not get cwd")?;
        String::from(name)
    };
    write!(
        output,
        "{}",
        toml::to_string(&config).map_err(|_| format!("Could not write to {}", &path))?
    )?;

    let path = format!("{}/{}/{}", name.unwrap_or("."), SRC, MAIN);
    let mut output = File::create(&path).map_err(|_| format!("Could not create {}", &path))?;
    write!(output, "{}", HELLO).map_err(|_| format!("Could not write to {}", &path))?;

    println!("\tCreated `{}` package", config.package.name);

    Command::new(GIT)
        .arg("init")
        .current_dir(name.unwrap_or("."))
        .output()
        .map_err(|_| "Could not run git to init repo")?;

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

        let path = format!("{}/{}.bas", SRC, filename);
        let f = File::open(&path).map_err(|_| format!("Could not open {}", &path))?;

        for line in BufReader::new(f).lines() {
            lines.push(line?)
        }
    }

    Ok(lines)
}

fn show_usage(action: Option<Action>) {
    println!("BASIC build system and package manager\n");
    match action {
        Some(Action::NEW) => println!("Usage: bargo new <name>\n"),
        Some(Action::UNKNOWN) => println!("Usage: bargo <new|build>\n"),
        None => println!("Usage: bargo <new|build>\n"),
    }
    println!("Commands:");
    println!("\tbuild\tBuild the current package");
    println!("\tclean\tRemove the generated file");
    println!("\temu\tRun the code inside an emulator");
    println!("\tinit\tCreate a new Bargo package in an existing directory");
    println!("\tnew\tCreate a new Bargo package")
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.get(0) {
        Some(action) => match action.as_str() {
            "build" => {
                if let Err(error) = build() {
                    eprintln!("{error}");
                }
            }
            "clean" => {
                if let Err(error) = clean() {
                    eprintln!("{error}")
                }
            }
            "emulator" | "emu" => {
                if let Err(error) = emulator() {
                    eprintln!("{error}")
                }
            }
            "init" => {
                if let Err(error) = new(None) {
                    eprintln!("{error}");
                }
            }
            "new" => match args.get(1) {
                Some(name) => {
                    if let Err(error) = new(Some(&name)) {
                        eprintln!("{error}");
                    }
                }
                None => show_usage(Some(Action::NEW)),
            },
            _ => show_usage(Some(Action::UNKNOWN)),
        },
        None => show_usage(None),
    }
}
