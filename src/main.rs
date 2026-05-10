use std::{
    fs::File,
    io::{self, Write},
    process,
};

mod command;
mod state;
mod tokenizer;

use command::Command;
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
        let mut args: Vec<&str> = tokens[1..].iter().map(|tk| tk.as_str()).collect();

        let mut out_output: Box<dyn Write> = Box::new(io::stdout());
        let mut err_output: Box<dyn Write> = Box::new(io::stderr());

        while let Some((idx, &token)) = args
            .iter()
            .enumerate()
            .find(|&(_, &token)| token == ">" || token == "1>" || token == "2>")
        {
            let Some(&filepath) = args.get(idx + 1) else {
                eprintln!("ERROR: no redirection target");
                continue;
            };
            let Ok(file) = File::create(filepath) else {
                eprintln!("ERROR: failed to create a file");
                continue;
            };

            if token == ">" || token == "1>" {
                out_output = Box::new(file);
            } else {
                // token == "2>"
                err_output = Box::new(file);
            }

            args = args
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != idx && i != idx + 1)
                .map(|(_, &v)| v)
                .collect();
        }

        Command::from_str(command).exec(&mut app_state, args, out_output, err_output);
    }
}
