use rustyline::config::Config;
use rustyline::history::DefaultHistory;
use rustyline::{Editor, error::ReadlineError};

use std::cell::RefCell;
use std::process::{self, Stdio};
use std::rc::Rc;

mod command;
mod helper;
mod job;
mod parser;
mod state;
mod tokenizer;

use command::Command;
use helper::ShellHelper;
use parser::ParsedCommand;
use state::AppState;

use crate::parser::ParsedPipedCommands;

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

            if tokens.contains(&String::from("|")) {
                let ParsedPipedCommands {
                    command_list,
                    args_list,
                    mut stdout,
                    mut stderr,
                    run_in_background: _,
                } = parser::parse_piped_commands(&tokens)?;

                // For now, we have exactly two commands
                let app_state = app_state.borrow();
                let first_command = process::Command::new(command_list[0].as_str())
                    .current_dir(app_state.cwd())
                    .args(&args_list[0])
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed to execute process");
                let second_command = process::Command::new(command_list[1].as_str())
                    .current_dir(app_state.cwd())
                    .args(&args_list[1])
                    .stdin(first_command.stdout.unwrap())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed to execute process");

                let output = second_command.wait_with_output().expect("no output");

                let stdout_str = String::from_utf8_lossy(&output.stdout);
                if !stdout_str.is_empty() {
                    let _ = write!(stdout, "{}", stdout_str);
                }

                let stderr_str = String::from_utf8_lossy(&output.stderr);
                if !stderr_str.is_empty() {
                    let _ = write!(stderr, "{}", stderr_str);
                }
            } else {
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
