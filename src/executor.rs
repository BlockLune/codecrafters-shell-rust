use anyhow::{Context, Result};
use fork::{Fork, WEXITSTATUS, WIFEXITED, fork, waitpid};
use std::io::{self, Read, Write};
use std::process;
use std::process::Stdio;

use crate::command::{self, Command, CommandReturnType};
use crate::context::ShellContext;
use crate::parser::{ParsedCommand, ParsedInput};

pub fn execute(ctx: &mut ShellContext, parsed_input: ParsedInput) -> Result<()> {
    if parsed_input.run_in_background {
        if parsed_input.commands.len() != 1 {
            eprintln!("background pipelines not yet supported");
            return Ok(());
        }
        let cmd = &parsed_input.commands[0];
        if command::Command::is_builtin(&cmd.name) {
            eprintln!("background execution of builtins not yet supported");
            return Ok(());
        }

        let name = &cmd.name;
        let command_line = format!("{} {}", name, cmd.args.join(" "));

        if !ctx.external_executables().contains_key(name.as_str()) {
            println!("{}: command not found", name);
            return Ok(());
        }

        let child = process::Command::new(name)
            .current_dir(ctx.cwd().to_path_buf())
            .args(&cmd.args)
            .spawn()
            .expect("failed to spawn");
        let pid = child.id();
        let job_number = ctx.add_background_job(&command_line, child);
        println!("[{}] {}", job_number, pid);

        return Ok(());
    }

    // single builtin runs in parent process
    if parsed_input.commands.len() == 1 && Command::is_builtin(&parsed_input.commands[0].name) {
        let command = parsed_input.commands.into_iter().next().unwrap();
        let name = command.name.as_str();
        let args = command.args;
        let stdin = Box::new(io::stdin());
        let stdout: Box<dyn Write + Send> = match command.stdout_redirect {
            Some(file) => Box::new(file),
            None => Box::new(io::stdout()),
        };
        let stderr: Box<dyn Write + Send> = match command.stderr_redirect {
            Some(file) => Box::new(file),
            None => Box::new(io::stderr()),
        };
        match exec_builtin_parent(ctx, name, args, stdin, stdout, stderr) {
            CommandReturnType::Continue => (),
            CommandReturnType::Exit(exit_code) => process::exit(exit_code),
        }
        return Ok(());
    }

    exec_pipeline(ctx, parsed_input.commands)?;

    Ok(())
}

fn exec_pipeline(ctx: &mut ShellContext, commands: Vec<ParsedCommand>) -> Result<()> {
    let n = commands.len();

    // Option::take() enforces each pipe end is consumed exactly once at compile time -
    // a second .take() returns None, making double-use a visible bug rather than a silent fd leak
    let mut pipes: Vec<_> = (0..n - 1)
        .map(|_| {
            let (r, w) = io::pipe().context("failed to create pipe")?;
            Ok((Some(r), Some(w)))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut children = Vec::new();

    for (i, command) in commands.into_iter().enumerate() {
        let pipe_reader = if i > 0 { pipes[i - 1].0.take() } else { None };
        let pipe_writer = if i < n - 1 { pipes[i].1.take() } else { None };

        if Command::is_builtin(&command.name) {
            let stdin: Box<dyn Read + Send> = match pipe_reader {
                Some(r) => Box::new(r),
                None => Box::new(io::stdin()),
            };
            let stdout: Box<dyn Write + Send> = match (command.stdout_redirect, pipe_writer) {
                (Some(file), _) => Box::new(file),
                (None, Some(w)) => Box::new(w),
                (None, None) => Box::new(io::stdout()),
            };
            let stderr: Box<dyn Write + Send> = match command.stderr_redirect {
                Some(file) => Box::new(file),
                None => Box::new(io::stderr()),
            };

            match exec_builtin_child(ctx, &command.name, command.args, stdin, stdout, stderr) {
                CommandReturnType::Continue => continue,
                CommandReturnType::Exit(exit_code) => process::exit(exit_code),
            }
        } else {
            let stdin_cfg = match pipe_reader {
                Some(r) => Stdio::from(r),
                None => Stdio::inherit(),
            };
            let stdout_cfg = match (command.stdout_redirect, pipe_writer) {
                (Some(file), _) => Stdio::from(file),
                (None, Some(w)) => Stdio::from(w),
                (None, None) => Stdio::inherit(),
            };
            let stderr_cfg = match command.stderr_redirect {
                Some(file) => Stdio::from(file),
                None => Stdio::inherit(),
            };

            match process::Command::new(&command.name)
                .current_dir(ctx.cwd())
                .args(&command.args)
                .stdin(stdin_cfg)
                .stdout(stdout_cfg)
                .stderr(stderr_cfg)
                .spawn()
            {
                Ok(child) => children.push(child),
                Err(_) => {
                    eprintln!("{}: command not found", command.name);
                }
            }
        }
    }

    for mut child in children {
        let _ = child.wait();
    }

    Ok(())
}

fn exec_builtin_parent(
    ctx: &mut ShellContext,
    name: &str,
    args: Vec<String>,
    stdin: Box<dyn Read + Send>,
    stdout: Box<dyn Write + Send>,
    stderr: Box<dyn Write + Send>,
) -> CommandReturnType {
    Command::from_str(name).exec(ctx, args, stdin, stdout, stderr)
}

// builtins in a pipeline must fork: their stdout needs to feed the next pipe segment,
// but fork() copies process memory so the child still has ShellContext for execution
fn exec_builtin_child(
    ctx: &mut ShellContext,
    name: &str,
    args: Vec<String>,
    stdin: Box<dyn Read + Send>,
    stdout: Box<dyn Write + Send>,
    stderr: Box<dyn Write + Send>,
) -> CommandReturnType {
    match fork() {
        Ok(Fork::Parent(child)) => match waitpid(child) {
            Ok(status) => {
                if WIFEXITED(status) {
                    let code = WEXITSTATUS(status);
                    if code == 0 {
                        CommandReturnType::Continue
                    } else {
                        CommandReturnType::Exit(code)
                    }
                } else {
                    eprintln!("child failed to exit normally");
                    CommandReturnType::Exit(1)
                }
            }
            Err(e) => {
                eprintln!("waitpid failed: {}", e);
                CommandReturnType::Exit(1)
            }
        },
        Ok(Fork::Child) => match exec_builtin_parent(ctx, name, args, stdin, stdout, stderr) {
            CommandReturnType::Continue => process::exit(0),
            CommandReturnType::Exit(code) => process::exit(code),
        },
        Err(e) => {
            eprintln!("Fork failed: {}", e);
            CommandReturnType::Exit(1)
        }
    }
}
