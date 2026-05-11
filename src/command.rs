use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process;

use crate::state::AppState;

pub enum Command<'a> {
    BuiltinExit,
    BuiltinEcho,
    BuiltinType,
    BuiltinPwd,
    BuiltinCd,
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
            external_command => Self::External(external_command),
        }
    }

    pub fn exec(
        &self,
        app_state: &mut AppState,
        args: Vec<&str>,
        out_output: Box<dyn Write>,
        err_output: Box<dyn Write>,
    ) {
        match self {
            Self::BuiltinExit => exit_command(app_state, args, out_output, err_output),
            Self::BuiltinEcho => echo_command(app_state, args, out_output, err_output),
            Self::BuiltinType => type_command(app_state, args, out_output, err_output),
            Self::BuiltinPwd => pwd_command(app_state, args, out_output, err_output),
            Self::BuiltinCd => cd_command(app_state, args, out_output, err_output),
            Self::External(command) => {
                exec_external(app_state, command, args, out_output, err_output)
            }
        }
    }
}

fn exit_command(
    _app_state: &mut AppState,
    args: Vec<&str>,
    mut out_output: Box<dyn Write>,
    mut err_output: Box<dyn Write>,
) {
    let _ = writeln!(out_output, "exit");

    if args.is_empty() {
        process::exit(0);
    } else if args.len() >= 2 {
        let _ = writeln!(err_output, "exit: too many arguments");
    } else {
        let _ = match args[0].parse::<i32>() {
            Ok(ret) => process::exit(ret),
            Err(_) => writeln!(err_output, "exit: {}: numeric argument required", args[0]),
        };
    }
}

fn echo_command(
    _app_state: &mut AppState,
    args: Vec<&str>,
    mut out_output: Box<dyn Write>,
    mut _err_output: Box<dyn Write>,
) {
    let _ = writeln!(out_output, "{}", args.join(" "));
}

fn type_command(
    app_state: &mut AppState,
    args: Vec<&str>,
    mut out_output: Box<dyn Write>,
    mut _err_output: Box<dyn Write>,
) {
    for command in args {
        if Command::is_builtin(command) {
            let _ = writeln!(out_output, "{} is a shell builtin", command);
        } else {
            let executables = app_state.get_external_executables();
            if executables.contains_key(command) {
                let _ = writeln!(
                    out_output,
                    "{} is {}",
                    command,
                    executables.get(command).unwrap().display()
                );
            } else {
                let _ = writeln!(out_output, "{}: not found", command);
            }
        }
    }
}

fn pwd_command(
    app_state: &mut AppState,
    _args: Vec<&str>,
    mut out_output: Box<dyn Write>,
    mut _err_output: Box<dyn Write>,
) {
    let path = app_state.get_cwd();
    let _ = writeln!(out_output, "{}", path.display());
}

fn cd_command(
    app_state: &mut AppState,
    args: Vec<&str>,
    mut _out_output: Box<dyn Write>,
    mut err_output: Box<dyn Write>,
) {
    if args.len() > 1 {
        let _ = writeln!(err_output, "cd: too many arguments");
        return;
    }

    let path = if args.len() == 0 || args[0] == "~" {
        PathBuf::from(env::var("HOME").unwrap())
    } else {
        PathBuf::from(args[0])
    };

    let _ = app_state
        .cd(path.clone())
        .map_err(|e| writeln!(err_output, "cd: {}: {}", path.display(), e));
}

fn exec_external(
    app_state: &AppState,
    command: &str,
    args: Vec<&str>,
    mut out_output: Box<dyn Write>,
    mut err_output: Box<dyn Write>,
) {
    if !app_state.get_external_executables().contains_key(command) {
        let _ = writeln!(out_output, "{}: command not found", command);
        return;
    }

    let output = process::Command::new(command)
        .current_dir(app_state.get_cwd())
        .args(args)
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        let _ = write!(out_output, "{}", stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        let _ = write!(err_output, "{}", stderr);
    }
}
