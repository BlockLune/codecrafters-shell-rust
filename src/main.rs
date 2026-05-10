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
        if let Err(e) = one_turn(&mut app_state) {
            eprintln!("ERROR: {}", e);
        }
    }
}

fn one_turn(app_state: &mut AppState) -> Result<(), String> {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.pop(); // remove line break

    let tokens = tokenizer::tokenize(&input)?;
    if tokens.is_empty() {
        return Ok(());
    }

    let ParsedCommand {
        command,
        args,
        stdout,
        stderr,
    } = parser::parse_command(&tokens)?;

    Command::from_str(command).exec(app_state, args, stdout, stderr);
    Ok(())
}
