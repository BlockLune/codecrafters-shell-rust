use crate::context::ShellContext;

mod command;
mod helper;
mod job;
mod parser;
mod context;
mod tokenizer;
mod executor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shell = ShellContext::new()?;
    shell.run();
    Ok(())
}
