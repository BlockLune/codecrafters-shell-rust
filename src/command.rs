use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};

use crate::job::Job;
use crate::state::AppState;

pub const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd", "complete", "jobs"];

pub enum Command<'a> {
    BuiltinExit,
    BuiltinEcho,
    BuiltinType,
    BuiltinPwd,
    BuiltinCd,
    BuiltinComplete,
    BuiltinJobs,
    External(&'a str),
}

impl<'a> Command<'a> {
    pub fn is_builtin(s: &'a str) -> bool {
        match Self::from_str(s) {
            Self::External(_) => false,
            _ => true,
        }
    }

    pub fn from_str(s: &'a str) -> Self {
        match s {
            "exit" => Self::BuiltinExit,
            "echo" => Self::BuiltinEcho,
            "type" => Self::BuiltinType,
            "pwd" => Self::BuiltinPwd,
            "cd" => Self::BuiltinCd,
            "complete" => Self::BuiltinComplete,
            "jobs" => Self::BuiltinJobs,
            external_command => Self::External(external_command),
        }
    }

    pub fn exec(
        &self,
        app_state: Arc<Mutex<AppState>>,
        args: Vec<&str>,
        stdin: Box<dyn Read + Send>,
        stdout: Box<dyn Write + Send>,
        stderr: Box<dyn Write + Send>,
    ) {
        match self {
            Self::BuiltinExit => exit_command(app_state, args, stdin, stdout, stderr),
            Self::BuiltinEcho => echo_command(app_state, args, stdin, stdout, stderr),
            Self::BuiltinType => type_command(app_state, args, stdin, stdout, stderr),
            Self::BuiltinPwd => pwd_command(app_state, args, stdin, stdout, stderr),
            Self::BuiltinCd => cd_command(app_state, args, stdin, stdout, stderr),
            Self::BuiltinComplete => complete_command(app_state, args, stdin, stdout, stderr),
            Self::BuiltinJobs => jobs_command(app_state, args, stdin, stdout, stderr),
            Self::External(_) => {}
        }
    }
}

fn exit_command(
    _app_state: Arc<Mutex<AppState>>,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut stderr: Box<dyn Write + Send>,
) {
    let _ = writeln!(stdout, "exit");

    if args.is_empty() {
        process::exit(0);
    } else if args.len() >= 2 {
        let _ = writeln!(stderr, "exit: too many arguments");
    } else {
        let _ = match args[0].parse::<i32>() {
            Ok(ret) => process::exit(ret),
            Err(_) => writeln!(stderr, "exit: {}: numeric argument required", args[0]),
        };
    }
}

fn echo_command(
    _app_state: Arc<Mutex<AppState>>,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut _stderr: Box<dyn Write + Send>,
) {
    let _ = writeln!(stdout, "{}", args.join(" "));
}

fn type_command(
    app_state: Arc<Mutex<AppState>>,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut _stderr: Box<dyn Write + Send>,
) {
    for command in args {
        if Command::is_builtin(command) {
            let _ = writeln!(stdout, "{} is a shell builtin", command);
        } else {
            let app_state_locked = app_state.lock().unwrap();
            let executables = app_state_locked.external_executables();
            if executables.contains_key(command) {
                let _ = writeln!(
                    stdout,
                    "{} is {}",
                    command,
                    executables.get(command).unwrap().display()
                );
            } else {
                let _ = writeln!(stdout, "{}: not found", command);
            }
        }
    }
}

fn pwd_command(
    app_state: Arc<Mutex<AppState>>,
    _args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut _stderr: Box<dyn Write + Send>,
) {
    let app_state_locked = app_state.lock().unwrap();
    let path = app_state_locked.cwd();
    let _ = writeln!(stdout, "{}", path.display());
}

fn cd_command(
    app_state: Arc<Mutex<AppState>>,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut _stdout: Box<dyn Write + Send>,
    mut stderr: Box<dyn Write + Send>,
) {
    if args.len() > 1 {
        let _ = writeln!(stderr, "cd: too many arguments");
        return;
    }

    let path = if args.len() == 0 || args[0] == "~" {
        PathBuf::from(env::var("HOME").unwrap())
    } else {
        PathBuf::from(args[0])
    };

    let mut app_state_locked = app_state.lock().unwrap();
    let _ = app_state_locked
        .cd(path.clone())
        .map_err(|e| writeln!(stderr, "cd: {}: {}", path.display(), e));
}

fn complete_command(
    app_state: Arc<Mutex<AppState>>,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut stderr: Box<dyn Write + Send>,
) {
    let mut print_flag = false;
    let mut unregister_flag = false;
    let mut completer_path: Option<PathBuf> = None;
    let mut names: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i];
        if arg == "-p" {
            print_flag = true;
        } else if arg == "-C" {
            if i + 1 >= args.len() {
                let _ = writeln!(stderr, "complete: -C: option requires an argument");
                return;
            }
            completer_path = Some(PathBuf::from(args[i + 1]));

            i += 1;
        } else if arg == "-r" {
            unregister_flag = true;
        } else {
            names.push(arg.to_string());
        }

        i += 1;
    }

    for name in names {
        let mut app_state_locked = app_state.lock().unwrap();
        if print_flag {
            if let Some(completer_path) = app_state_locked.get_completer(&name) {
                let _ = writeln!(
                    stdout,
                    "complete -C '{}' {}",
                    completer_path.display(),
                    name
                );
            } else {
                let _ = writeln!(stderr, "complete: {}: no completion specification", name);
            }
        } else if unregister_flag {
            app_state_locked.unregister_completion(name);
        } else {
            // Or use: `if let Some(ref path) = completer_path`
            // where `ref` indicates: use borrow in pattern matching, instead of move
            if let Some(path) = completer_path.as_ref() {
                app_state_locked.register_completion(name, path.clone());
            }
        }
    }
}

fn jobs_command(
    app_state: Arc<Mutex<AppState>>,
    _args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut _stderr: Box<dyn Write + Send>,
) {
    let mut app_state_locked = app_state.lock().unwrap();
    let jobs = app_state_locked.jobs();
    let statuses = Job::compute_job_status(jobs);

    for (job, entry) in jobs.iter_mut().zip(statuses.iter()) {
        let Some((indicator, status)) = entry else {
            continue;
        };

        if status == "Done" {
            job.done = true;
        }

        let _ = writeln!(stdout, "{}", job.display(indicator, status));
    }

    jobs.retain(|job| !job.done);
}
