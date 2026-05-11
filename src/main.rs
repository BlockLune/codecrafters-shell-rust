use rustyline::config::Config;
use rustyline::history::DefaultHistory;
use rustyline::{Editor, error::ReadlineError};

mod command;
mod helper;
mod parser;
mod state;
mod tokenizer;

use command::Command;
use helper::ShellHelper;
use parser::ParsedCommand;
use state::AppState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app_state = AppState::default()?;
    let shell_helper = ShellHelper::new(&app_state);
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(shell_helper));

    // REPL (Read-Eval-Print Loop)
    loop {
        if let Err(e) = one_turn(&mut app_state, &mut rl) {
            eprintln!("Error: {}", e);
        }
    }
}

fn one_turn(
    app_state: &mut AppState,
    rl: &mut Editor<ShellHelper, DefaultHistory>,
) -> Result<(), String> {
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
