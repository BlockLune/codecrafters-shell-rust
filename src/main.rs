use rustyline::config::Config;
use rustyline::history::DefaultHistory;
use rustyline::{Editor, error::ReadlineError};

use std::cell::RefCell;
use std::rc::Rc;

mod command;
mod helper;
mod parser;
mod state;
mod tokenizer;
mod job;

use command::Command;
use helper::ShellHelper;
use parser::ParsedCommand;
use state::AppState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Rc::new(RefCell::new(AppState::default()?));
    let shell_helper = ShellHelper::new(Rc::clone(&app_state));
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(shell_helper));

    // REPL (Read-Eval-Print Loop)
    loop {
        if let Err(e) = one_turn(Rc::clone(&app_state), &mut rl) {
            eprintln!("Error: {}", e);
        }
    }
}

fn one_turn(
    app_state: Rc<RefCell<AppState>>,
    rl: &mut Editor<ShellHelper, DefaultHistory>,
) -> Result<(), String> {
    app_state.borrow_mut().reap_done_jobs();

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
                run_in_background,
            } = parser::parse_command(&tokens)?;

            Command::from_str(command).exec(
                Rc::clone(&app_state),
                args,
                run_in_background,
                stdout,
                stderr,
            );
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
