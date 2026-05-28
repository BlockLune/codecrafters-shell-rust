use crate::context::ShellContext;

mod command;
mod helper;
mod job;
mod parser;
mod pipeline;
mod context;
mod tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shell = ShellContext::new()?;
    shell.run();
    Ok(())
}
