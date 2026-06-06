use anyhow::Result;

use crate::context::ShellContext;

mod command;
mod helper;
mod job;
mod parser;
mod context;
mod tokenizer;
mod executor;

fn main() -> Result<()> {
    let mut shell = ShellContext::new()?;
    shell.run();
    Ok(())
}
