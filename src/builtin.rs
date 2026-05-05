use std::process;

use super::exec::build_executables;

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

    if command == "exit" || command == "echo" || command == "type" {
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
