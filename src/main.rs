use bargo::commands::{
    AddCommand, BargoCommand, BuildCommand, CleanCommand, EmuCommand, NewCommand,
};
use std::env;

enum Action {
    Add,
    New,
    Unknown,
}

fn show_usage(action: Option<Action>) {
    println!("BASIC build system and package manager\n");
    match action {
        Some(Action::Add) => println!("Usage: bargo add <dependency>\n"),
        Some(Action::New) => println!("Usage: bargo new <name>\n"),
        Some(Action::Unknown) => println!("Usage: bargo <new|build>\n"),
        None => println!("Usage: bargo <new|build>\n"),
    }
    println!("Commands:");
    println!("{}", AddCommand::usage());
    println!("{}", BuildCommand::usage());
    println!("{}", CleanCommand::usage());
    println!("{}", EmuCommand::usage());
    println!("{}", NewCommand::usage());
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first() {
        Some(action) => match action.as_str() {
            "add" => match args.get(1) {
                Some(dependency) => match AddCommand::new(dependency) {
                    Ok(add_command) => {
                        if let Err(error) = add_command.execute() {
                            eprintln!("{error}")
                        }
                    }
                    Err(error) => eprintln!("{error}"),
                },
                None => show_usage(Some(Action::Add)),
            },
            "build" => match BuildCommand::new() {
                Ok(build_command) => {
                    if let Err(error) = build_command.execute() {
                        eprintln!("{error}")
                    }
                }
                Err(error) => eprintln!("{error}"),
            },
            "clean" => match CleanCommand::new() {
                Ok(clean_command) => {
                    if let Err(error) = clean_command.execute() {
                        eprintln!("{error}")
                    }
                }
                Err(error) => eprintln!("{error}"),
            },
            "emulator" | "emu" => match EmuCommand::new() {
                Ok(emu_command) => {
                    if let Err(error) = emu_command.execute() {
                        eprintln!("{error}")
                    }
                }
                Err(error) => eprintln!("{error}"),
            },
            "init" => {
                let new_command = NewCommand::new(None);

                if let Err(error) = new_command.execute() {
                    eprintln!("{error}");
                }
            }
            "new" => match args.get(1) {
                Some(name) => {
                    let new_command = NewCommand::new(Some(name));

                    if let Err(error) = new_command.execute() {
                        eprintln!("{error}");
                    }
                }
                None => show_usage(Some(Action::New)),
            },
            _ => show_usage(Some(Action::Unknown)),
        },
        None => show_usage(None),
    }
}
