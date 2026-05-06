use std::env;
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
            Self::BuiltinExit => true,
            Self::BuiltinEcho => true,
            Self::BuiltinType => true,
            Self::BuiltinPwd => true,
            Self::BuiltinCd => true,
            _ => false,
        }
    }

    pub fn from_str(s: &'a str) -> Self {
        match s {
            "exit" => Self::BuiltinExit,
            "echo" => Self::BuiltinEcho,
            "type" => Self::BuiltinType,
            "pwd" => Self::BuiltinPwd,
            "cd" => Self::BuiltinCd,
            external_command => Self::External(external_command)
        }
    }

    pub fn exec(&self, app_state: &mut AppState, args: Vec<&str>) {
        match self {
            Self::BuiltinExit => exit_command(app_state, args),
            Self::BuiltinEcho => echo_command(app_state, args),
            Self::BuiltinType => type_command(app_state, args),
            Self::BuiltinPwd => pwd_command(app_state, args),
            Self::BuiltinCd => cd_command(app_state, args),
            Self::External(command) => exec_external(app_state, command, args),
        }
    }
}

fn exit_command(_app_state: &mut AppState, _args: Vec<&str>) {
    process::exit(0);
}

fn echo_command(_app_state: &mut AppState, args: Vec<&str>) {
    println!("{}", args.join(" "));
}

fn type_command(app_state: &mut AppState, args: Vec<&str>) {
    for command in args {
        if Command::is_builtin(command) {
            println!("{} is a shell builtin", command);
        } else {
            let executables = app_state.get_external_executables();
            if executables.contains_key(command) {
                println!(
                    "{} is {}",
                    command,
                    executables.get(command).unwrap().display()
                )
            } else {
                println!("{}: not found", command);
            }
        }
    }
}

fn pwd_command(app_state: &mut AppState, _args: Vec<&str>) {
    let path = app_state.get_cwd();
    println!("{}", path.display());
}

fn cd_command(app_state: &mut AppState, args: Vec<&str>) {
    if args.len() > 1 {
        eprintln!("cd: too many arguments");
        return;
    }

    let path = if args.len() == 0 || args[0] == "~" {
        PathBuf::from(env::var("HOME").unwrap())
    } else {
        PathBuf::from(args[0])
    };

    let _ = app_state
        .cd(path.clone())
        .map_err(|e| eprintln!("cd: {}: {}", path.display(), e));
}

fn exec_external(app_state: &AppState, command: &str, args: Vec<&str>) {
    if !app_state.get_external_executables().contains_key(command) {
        println!("{}: command not found", command);
        return;
    }

    let output = process::Command::new(command)
        .current_dir(app_state.get_cwd())
        .args(args)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{}", stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }
}
