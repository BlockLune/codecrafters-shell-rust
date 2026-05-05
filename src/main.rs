use std::io::{self, Write};
use std::process;

mod builtin;
mod exec;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let commands: Vec<_> = command.split_whitespace().collect();

        match commands.first() {
            Some(&"exit") => builtin::exit_command(),
            Some(&"echo") => builtin::echo_command(&commands[1..].join(" ")),
            Some(&"type") => builtin::type_command(commands.get(1).copied()),
            Some(command) => exec_external(command, &commands[1..]),
            None => (),
        }
    }
}

fn exec_external(command: &str, args: &[&str]) {
    let executables = exec::build_executables();

    if !executables.contains_key(command) {
        println!("{}: command not found", command);
        return;
    }

    let output = process::Command::new(command)
        .args(args)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }
}
