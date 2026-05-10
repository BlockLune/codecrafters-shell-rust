use std::{
    io::{self, Write},
    process,
};

mod command;
mod parser;
mod state;
mod tokenizer;

use command::Command;
use parser::ParsedCommand;
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
        input.pop(); // remove line break

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

        let ParsedCommand {
            command,
            args,
            stdout,
            stderr,
        } = match parser::parse_command(&tokens) {
            Ok(parsed_command) => parsed_command,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                continue;
            }
        };

        Command::from_str(command).exec(&mut app_state, args, stdout, stderr);
    }
}
