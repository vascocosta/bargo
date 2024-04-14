use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::read_to_string;
use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, env, error::Error, fs::File, path::PathBuf};

const EMU: &str = "fab-agon-emulator";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub package: Package,
    pub dependencies: Option<HashMap<String, Option<String>>>,
}

impl Config {
    pub fn read<P>(path: P) -> Result<Config, Box<dyn Error>>
    where
        P: AsRef<Path> + Display,
    {
        let config =
            toml::from_str(&read_to_string(&path).map_err(|_| format!("Could not open {}", path))?)
                .map_err(|_| format!("Syntax error in {}", path))?;

        Ok(config)
    }

    pub fn write<P>(&self, path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path> + Display,
    {
        let mut output = File::create(&path).map_err(|_| format!("Could not create {}", path))?;
        write!(
            output,
            "{}",
            toml::to_string(&self).map_err(|_| format!("Could not write to {}", path))?
        )?;

        Ok(())
    }
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
pub struct Package {
    pub name: String,
    pub carriage_return: bool,
    pub numbering: usize,
    pub emu_path: PathBuf,
    pub version: String,
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
