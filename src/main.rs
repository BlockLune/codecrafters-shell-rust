use rustyline::config::Config;
use rustyline::history::DefaultHistory;
use rustyline::{Editor, error::ReadlineError};

use std::process;
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
                if parsed_input.commands.len() != 1 {
                    eprintln!("background pipelines not yet supported");
                    return Ok(());
                }
                let cmd = &parsed_input.commands[0];
                if command::Command::is_builtin(cmd.name) {
                    eprintln!("background execution of builtins not yet supported");
                    return Ok(());
                }

                let name = cmd.name.to_string();
                let args: Vec<String> = cmd.args.iter().map(|s| s.to_string()).collect();
                let command_line = format!("{} {}", name, args.join(" "));

                let mut state = app_state.lock().unwrap();
                if !state.external_executables().contains_key(name.as_str()) {
                    println!("{}: command not found", name);
                    return Ok(());
                }

                let child = process::Command::new(&name)
                    .current_dir(state.cwd())
                    .args(&args)
                    .spawn()
                    .expect("failed to spawn");
                let pid = child.id();
                let job_number = state.add_background_job(&command_line, child);
                println!("[{}] {}", job_number, pid);
            } else {
                pipeline::exec_pipeline(Arc::clone(&app_state), parsed_input.commands);
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
