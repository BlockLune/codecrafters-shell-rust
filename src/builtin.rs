use std::env;
use std::path::{Path, PathBuf};
use std::process;

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
                executables.get(command).unwrap().display()
            )
        } else {
            println!("{}: not found", command);
        }
    }
}

pub fn pwd_command(path: &Path) {
    println!("{}", path.display());
}

pub fn cd_command(path: Option<&str>, app_state: &mut AppState) {
    let home_path = PathBuf::from(env::var("HOME").unwrap());

    let path = match path {
        Some(path) => {
            if path.trim() == "" || path.trim() == "~" {
                home_path
            } else {
                PathBuf::from(path)
            }
        }
        None => home_path,
    };

    let _ = app_state
        .cd(path.clone())
        .map_err(|e| eprintln!("cd: {}: {}", path.display(), e));
}
