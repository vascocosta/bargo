use bargo::commands::{
    Action, BargoCommand, BuildCommand, CleanCommand, DepCommand, EmuCommand, NewCommand,
};
use std::env;

fn show_usage(action: Option<Action>) {
    println!("BASIC build system and package manager\n");

    match action {
        Some(Action::DepAdd) => println!("Usage: bargo add <DEP>\n"),
        Some(Action::New) => println!("Usage: bargo new <NAME>\n"),
        Some(Action::DepRemove) => println!("Usage: bargo remove <DEP>\n"),
        Some(Action::Unknown) => println!("Usage: [COMMAND]\n"),
        Some(_) => (),
        None => println!("Usage: [COMMAND]\n"),
    }

    println!("Commands:");
    println!("{}", DepCommand::usage(Some(Action::DepAdd)));
    println!("{}", BuildCommand::usage(None));
    println!("{}", CleanCommand::usage(None));
    println!("{}", EmuCommand::usage(None));
    println!("{}", NewCommand::usage(Some(Action::Init)));
    println!("{}", NewCommand::usage(Some(Action::New)));
    println!("{}", DepCommand::usage(Some(Action::DepRemove)));
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first() {
        Some(action) => match action.as_str() {
            "add" => match args.get(1) {
                Some(dependency) => match DepCommand::new(dependency, Action::DepAdd) {
                    Ok(add_command) => {
                        if let Err(error) = add_command.execute() {
                            eprintln!("{error}")
                        }
                    }
                    Err(error) => eprintln!("{error}"),
                },
                None => show_usage(Some(Action::DepAdd)),
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
            "remove" => match args.get(1) {
                Some(dependency) => match DepCommand::new(dependency, Action::DepRemove) {
                    Ok(remove_command) => {
                        if let Err(error) = remove_command.execute() {
                            eprintln!("{error}")
                        }
                    }
                    Err(error) => eprintln!("{error}"),
                },
                None => show_usage(Some(Action::DepRemove)),
            },
            _ => show_usage(Some(Action::Unknown)),
        },
        None => show_usage(None),
    }
}
