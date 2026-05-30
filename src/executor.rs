use std::io::{self, Read, Write};
use std::process;
use std::process::Stdio;

use crate::command::{self, Command, CommandReturnType};
use crate::context::ShellContext;
use crate::parser::{ParsedCommand, ParsedInput};

pub fn execute(ctx: &mut ShellContext, parsed_input: ParsedInput) -> Result<(), String> {
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
    } else {
        exec_pipeline(ctx, parsed_input.commands);
    }

    Ok(())
}

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
            match Command::from_str(&command.name).exec(ctx, command.args, stdin, stdout, stderr) {
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
}
