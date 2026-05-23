use std::cell::RefCell;
use std::env;
use std::io::{Write, pipe};
use std::path::PathBuf;
use std::process::{self, Stdio};
use std::rc::Rc;

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
        app_state: Rc<RefCell<AppState>>,
        args: Vec<&str>,
        run_in_background: bool,
        stdout: Box<dyn Write>,
        stderr: Box<dyn Write>,
    ) {
        match self {
            Self::BuiltinExit => exit_command(app_state, args, stdout, stderr),
            Self::BuiltinEcho => echo_command(app_state, args, stdout, stderr),
            Self::BuiltinType => type_command(app_state, args, stdout, stderr),
            Self::BuiltinPwd => pwd_command(app_state, args, stdout, stderr),
            Self::BuiltinCd => cd_command(app_state, args, stdout, stderr),
            Self::BuiltinComplete => complete_command(app_state, args, stdout, stderr),
            Self::BuiltinJobs => jobs_command(app_state, args, stdout, stderr),
            Self::External(command) => {
                exec_external(app_state, command, args, run_in_background, stdout, stderr)
            }
        }
    }
}

fn exit_command(
    _app_state: Rc<RefCell<AppState>>,
    args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut stderr: Box<dyn Write>,
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
    _app_state: Rc<RefCell<AppState>>,
    args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut _stderr: Box<dyn Write>,
) {
    let _ = writeln!(stdout, "{}", args.join(" "));
}

fn type_command(
    app_state: Rc<RefCell<AppState>>,
    args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut _stderr: Box<dyn Write>,
) {
    for command in args {
        if Command::is_builtin(command) {
            let _ = writeln!(stdout, "{} is a shell builtin", command);
        } else {
            let app_state = app_state.borrow();
            let executables = app_state.external_executables();
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
    app_state: Rc<RefCell<AppState>>,
    _args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut _stderr: Box<dyn Write>,
) {
    let app_state = app_state.borrow();
    let path = app_state.cwd();
    let _ = writeln!(stdout, "{}", path.display());
}

fn cd_command(
    app_state: Rc<RefCell<AppState>>,
    args: Vec<&str>,
    mut _stdout: Box<dyn Write>,
    mut stderr: Box<dyn Write>,
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

    let mut app_state = app_state.borrow_mut();
    let _ = app_state
        .cd(path.clone())
        .map_err(|e| writeln!(stderr, "cd: {}: {}", path.display(), e));
}

fn complete_command(
    app_state: Rc<RefCell<AppState>>,
    args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut stderr: Box<dyn Write>,
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
        if print_flag {
            let app_state = app_state.borrow();
            if let Some(completer_path) = app_state.get_completer(&name) {
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
            let mut app_state = app_state.borrow_mut();
            app_state.unregister_completion(name);
        } else {
            // Or use: `if let Some(ref path) = completer_path`
            // where `ref` indicates: use borrow in pattern matching, instead of move
            if let Some(path) = completer_path.as_ref() {
                let mut app_state = app_state.borrow_mut();
                app_state.register_completion(name, path.clone());
            }
        }
    }
}

fn jobs_command(
    app_state: Rc<RefCell<AppState>>,
    _args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut _stderr: Box<dyn Write>,
) {
    let mut app_state = app_state.borrow_mut();
    let jobs = app_state.jobs();
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

fn exec_external(
    app_state: Rc<RefCell<AppState>>,
    command: &str,
    args: Vec<&str>,
    run_in_background: bool,
    mut stdout: Box<dyn Write>,
    mut stderr: Box<dyn Write>,
) {
    let mut app_state = app_state.borrow_mut();

    if !app_state.external_executables().contains_key(command) {
        let _ = writeln!(stdout, "{}: command not found", command);
        return;
    }

    if run_in_background {
        let child = process::Command::new(command)
            .current_dir(app_state.cwd())
            .args(args.clone())
            .spawn()
            .expect("failed to execute process");
        let pid = child.id();
        let job_number =
            app_state.add_background_job(format!("{} {}", command, args.join(" ")).as_str(), child);
        let _ = writeln!(stdout, "[{}] {}", job_number, pid);
        return;
    }

    let output = process::Command::new(command)
        .current_dir(app_state.cwd())
        .args(args)
        .output()
        .expect("failed to execute process");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    if !stdout_str.is_empty() {
        let _ = write!(stdout, "{}", stdout_str);
    }

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    if !stderr_str.is_empty() {
        let _ = write!(stderr, "{}", stderr_str);
    }
}

pub fn exec_external_pipelined(app_state: Rc<RefCell<AppState>>, segments: &[&[String]]) {
    let app_state = app_state.borrow();
    let cwd = app_state.cwd();

    let seg1 = segments[0];
    let seg2 = segments[1];

    let cmd1 = &seg1[0].as_str();
    let args1: Vec<&str> = seg1[1..].iter().map(|s| s.as_str()).collect();
    let cmd2 = &seg2[0].as_str();
    let args2: Vec<&str> = seg2[1..].iter().map(|s| s.as_str()).collect();

    let (pipe_reader, pipe_writer) = pipe().expect("failed to create pipe");

    let mut child1 = process::Command::new(cmd1)
        .current_dir(cwd)
        .args(&args1)
        .stdout(Stdio::from(pipe_writer))
        .spawn()
        .expect("failed to spawn the first command");

    let mut child2 = process::Command::new(cmd2)
        .current_dir(cwd)
        .args(&args2)
        .stdin(Stdio::from(pipe_reader))
        .spawn()
        .expect("failed to spawn the second command");

    let _ = child2.wait();
    let _ = child1.wait();
}
