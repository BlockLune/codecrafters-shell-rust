use std::env;
use std::path::PathBuf;
use std::process;

use crate::exec::build_executables;
use crate::state::AppState;

pub const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

pub fn is_builtin(command: &str) -> bool {
    BUILTIN_COMMANDS.contains(&command)
}

pub fn exit_command(_app_state: &mut AppState, _args: Vec<&str>) {
    process::exit(0);
}

pub fn echo_command(_app_state: &mut AppState, args: Vec<&str>) {
    println!("{}", args.join(" "));
}

pub fn type_command(_app_state: &mut AppState, args: Vec<&str>) {
    for command in args {
        if is_builtin(command) {
            println!("{} is a shell builtin", command);
        } else {
            let executables = build_executables();
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

pub fn pwd_command(app_state: &mut AppState, _args: Vec<&str>) {
    let path = app_state.get_cwd();
    println!("{}", path.display());
}

pub fn cd_command(app_state: &mut AppState, args: Vec<&str>) {
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
