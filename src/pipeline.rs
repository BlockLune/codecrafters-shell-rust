use std::io::{self, Read, Write};
use std::process;
use std::process::Stdio;

use crate::command::Command;
use crate::parser::ParsedCommand;
use crate::context::ShellContext;

pub fn exec_pipeline(ctx: &mut ShellContext, commands: Vec<ParsedCommand>) {
    let n = commands.len();

    let mut pipes: Vec<_> = (0..n - 1)
        .map(|_| {
            let (r, w) = io::pipe().expect("failed to create pipe");
            (Some(r), Some(w))
        })
        .collect();

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

            // TODO: thread?
            Command::from_str(command.name).exec(ctx, command.args, stdin, stdout, stderr);
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
}
