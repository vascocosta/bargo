use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, path::PathBuf};

const EMU: &str = "fab-agon-emulator";

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub package: Package,
    pub dependencies: Option<HashMap<String, String>>,
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
