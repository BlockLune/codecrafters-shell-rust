use std::process;
use std::path::PathBuf;
use std::env;

use crate::exec::build_executables;
use crate::state::AppState;

pub const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd"];

pub fn is_builtin(command: &str) -> bool {
    BUILTIN_COMMANDS.contains(&command)
}

pub fn exit_command() {
    process::exit(0);
}

pub fn echo_command(s: &str) {
    println!("{}", s);
}

pub fn type_command(command: Option<&str>) {
    if command.is_none() {
        return;
    }
    let command = command.unwrap();

    if is_builtin(command) {
        println!("{} is a shell builtin", command);
    } else {
        let executables = build_executables();
        if executables.contains_key(command) {
            println!(
                "{} is {}",
                command,
                executables
                    .get(command)
                    .unwrap()
                    .display()
            )
        } else {
            println!("{}: not found", command);
        }
    }
}

pub fn pwd_command(path: Option<&PathBuf>) {
    println!("{}", path.unwrap().display());
}

pub fn cd_command(path: Option<&str>, app_state: &mut AppState) {
    if path.is_none() {
        app_state.cd(PathBuf::from(env::var("HOME").unwrap()));
        return;
    }

    let path = PathBuf::from(path.unwrap());
    app_state.cd(path);
}
