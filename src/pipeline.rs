use std::io::{self, Read, Write};
use std::process;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::command::Command;
use crate::parser::ParsedCommand;
use crate::state::AppState;

pub fn exec_pipeline(app_state: Arc<Mutex<AppState>>, commands: Vec<ParsedCommand>) {
    let n = commands.len();

    let mut pipes: Vec<_> = (0..n - 1)
        .map(|_| {
            let (r, w) = io::pipe().expect("failed to create pipe");
            (Some(r), Some(w))
        })
        .collect();

    let mut handles = Vec::new();
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

            let state = Arc::clone(&app_state);
            let name = command.name.to_string();
            let args: Vec<String> = command.args.into_iter().map(|s| s.to_string()).collect();

            handles.push(thread::spawn(move || {
                let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                Command::from_str(&name).exec(state, args_ref, stdin, stdout, stderr);
            }));
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

            let child = process::Command::new(command.name)
                .current_dir(app_state.lock().unwrap().cwd())
                .args(&command.args)
                .stdin(stdin_cfg)
                .stdout(stdout_cfg)
                .stderr(stderr_cfg)
                .spawn()
                .expect("failed to spawn child");
            children.push(child);
        }
    }

    for handle in handles {
        let _ = handle.join();
    }
    for mut child in children {
        let _ = child.wait();
    }
}
