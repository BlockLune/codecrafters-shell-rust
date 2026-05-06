use std::{
    io::{self, Write},
    process,
};

mod builtin;
mod exec;
mod state;
mod tokenizer;

use state::AppState;

fn main() {
    let mut app_state = AppState::default().unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        process::exit(1);
    });

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let tokens = match tokenizer::tokenize(&input) {
            Ok(tks) => tks,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                continue;
            }
        };

        if tokens.is_empty() {
            continue;
        }

        let command = tokens.first().unwrap().as_str();
        let args: Vec<&str> = tokens[1..].iter().map(|tk| tk.as_str()).collect();

        if builtin::is_builtin(command) {
            match command {
                "exit" => builtin::exit_command(&mut app_state, args),
                "echo" => builtin::echo_command(&mut app_state, args),
                "type" => builtin::type_command(&mut app_state, args),
                "pwd" => builtin::pwd_command(&mut app_state, args),
                "cd" => builtin::cd_command(&mut app_state, args),
                _ => (),
            }
        } else {
            exec::exec_external(&app_state, command, args)
        }
    }
}
