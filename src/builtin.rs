use std::process;
use std::env;

use super::exec::build_executables;

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
                    .to_string_lossy()
                    .to_string()
            )
        } else {
            println!("{}: not found", command);
        }
    }
}

pub fn pwd_command() {
    if let Ok(path) = env::current_dir() {
        println!("{}", path.to_string_lossy().to_string());
    }
}
