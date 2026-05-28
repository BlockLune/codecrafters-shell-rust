use crate::state::AppState;

mod command;
mod helper;
mod job;
mod parser;
mod pipeline;
mod state;
mod tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shell = AppState::new()?;
    shell.run();
    Ok(())
}
