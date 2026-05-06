use std::{
    io::{self, Write},
    process,
};

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

        let command = tokens.first().unwrap().as_str();
        let args: Vec<&str> = tokens[1..].iter().map(|tk| tk.as_str()).collect();
        exec::Command::from_str(command).exec(&mut app_state, args);
    }
}
