use rustyline::{Editor, error::ReadlineError};
use rustyline::history::DefaultHistory;
use std::process;

mod command;
mod parser;
mod state;
mod tokenizer;
mod helper;

use command::Command;
use parser::ParsedCommand;
use state::AppState;
use helper::ShellHelper;

fn main() {
    let mut app_state = AppState::default().unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        process::exit(1);
    });
    let shell_helper = ShellHelper::new();
    let mut rl = Editor::new().unwrap_or_else(|e| {
        eprintln!("ERROR: {}", e);
        process::exit(1);
    });
    rl.set_helper(Some(shell_helper));

    // REPL (Read-Eval-Print Loop)
    loop {
        if let Err(e) = one_turn(&mut app_state, &mut rl) {
            eprintln!("ERROR: {}", e);
        }
    }
}

fn one_turn(app_state: &mut AppState, rl: &mut Editor<ShellHelper, DefaultHistory>) -> Result<(), String> {
    match rl.readline("$ ") {
        Ok(input) => {
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
        Err(ReadlineError::Interrupted) => {
            println!("^C");
            Ok(())
        }
        Err(ReadlineError::Eof) => {
            println!(r#"Use "exit" to leave the shell."#);
            Ok(())
        }
        Err(e) => Err(format!("readline error: {}", e)),
    }
}
