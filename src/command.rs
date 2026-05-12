use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process;

use crate::state::AppState;

pub const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd", "complete"];

pub enum Command<'a> {
    BuiltinExit,
    BuiltinEcho,
    BuiltinType,
    BuiltinPwd,
    BuiltinCd,
    BuiltinComplete,
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
            external_command => Self::External(external_command),
        }
    }

    pub fn exec(
        &self,
        app_state: &mut AppState,
        args: Vec<&str>,
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
            Self::External(command) => {
                exec_external(app_state, command, args, stdout, stderr)
            }
        }
    }
}

fn exit_command(
    _app_state: &mut AppState,
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
    _app_state: &mut AppState,
    args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut _stderr: Box<dyn Write>,
) {
    let _ = writeln!(stdout, "{}", args.join(" "));
}

fn type_command(
    app_state: &mut AppState,
    args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut _stderr: Box<dyn Write>,
) {
    for command in args {
        if Command::is_builtin(command) {
            let _ = writeln!(stdout, "{} is a shell builtin", command);
        } else {
            let executables = app_state.get_external_executables();
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
    app_state: &mut AppState,
    _args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut _stderr: Box<dyn Write>,
) {
    let path = app_state.get_cwd();
    let _ = writeln!(stdout, "{}", path.display());
}

fn cd_command(
    app_state: &mut AppState,
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

    let _ = app_state
        .cd(path.clone())
        .map_err(|e| writeln!(stderr, "cd: {}: {}", path.display(), e));
}

fn complete_command(
    _app_state: &mut AppState,
    args: Vec<&str>,
    mut _stdout: Box<dyn Write>,
    mut stderr: Box<dyn Write>,
) {
    if let Some((idx, _)) = args.iter().enumerate().find(|&(_, &arg)| arg == "-p") {
        if idx + 1 < args.len() {
            let program = args[idx + 1];
            let _ = writeln!(stderr, "complete: {}: no completion specification", program);
        }
    }
}

fn exec_external(
    app_state: &AppState,
    command: &str,
    args: Vec<&str>,
    mut stdout: Box<dyn Write>,
    mut stderr: Box<dyn Write>,
) {
    if !app_state.get_external_executables().contains_key(command) {
        let _ = writeln!(stdout, "{}: command not found", command);
        return;
    }

    let output = process::Command::new(command)
        .current_dir(app_state.get_cwd())
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
