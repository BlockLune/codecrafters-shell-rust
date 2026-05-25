use rustyline::config::Config;
use rustyline::history::DefaultHistory;
use rustyline::{Editor, error::ReadlineError};

use std::sync::{Arc, Mutex};

mod command;
mod helper;
mod job;
mod parser;
mod state;
mod tokenizer;
mod pipeline;

use helper::ShellHelper;
use state::AppState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(Mutex::new(AppState::default()?));
    let shell_helper = ShellHelper::new(Arc::clone(&app_state));
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(shell_helper));

    // REPL (Read-Eval-Print Loop)
    loop {
        if let Err(e) = one_turn(Arc::clone(&app_state), &mut rl) {
            eprintln!("Error: {}", e);
        }
    }
}

fn one_turn(
    app_state: Arc<Mutex<AppState>>,
    rl: &mut Editor<ShellHelper, DefaultHistory>,
) -> Result<(), String> {
    app_state.lock().unwrap().reap_done_jobs();

    match rl.readline("$ ") {
        Ok(input) => {
            let tokens = tokenizer::tokenize(&input)?;
            let parsed_input = parser::parse_input(&tokens)?;

            if parsed_input.run_in_background {
                // TODO: run in background
            } else {
                pipeline::exec_pipeline(app_state, parsed_input.commands);
            }

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
