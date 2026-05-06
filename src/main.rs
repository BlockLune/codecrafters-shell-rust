use std::{
    io::{self, Write},
    process,
};

mod builtin;
mod exec;
mod state;

use state::AppState;

fn main() {
    let mut app_state = AppState::default().unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        process::exit(1);
    });

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
            Some(&"pwd") => builtin::pwd_command(app_state.get_cwd()),
            Some(&"cd") => builtin::cd_command(commands.get(1).copied(), &mut app_state),
            Some(command) => exec::exec_external(command, &commands[1..], &app_state),
            None => (),
        }
    }
}
