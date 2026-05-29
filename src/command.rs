use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process;

use rustyline::history::History;

use crate::context::ShellContext;
use crate::job::Job;

pub const BUILTIN_COMMANDS: &[&str] = &[
    "exit", "echo", "type", "pwd", "cd", "complete", "jobs", "history", "declare",
];

#[allow(unused)]
pub enum Command<'a> {
    BuiltinExit,
    BuiltinEcho,
    BuiltinType,
    BuiltinPwd,
    BuiltinCd,
    BuiltinComplete,
    BuiltinJobs,
    BuiltinHistory,
    BuiltinDeclare,
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
            "history" => Self::BuiltinHistory,
            "declare" => Self::BuiltinDeclare,
            external_command => Self::External(external_command),
        }
    }

    pub fn exec(
        &self,
        ctx: &mut ShellContext,
        args: Vec<&str>,
        stdin: Box<dyn Read + Send>,
        stdout: Box<dyn Write + Send>,
        stderr: Box<dyn Write + Send>,
    ) {
        match self {
            Self::BuiltinExit => exit_command(ctx, args, stdin, stdout, stderr),
            Self::BuiltinEcho => echo_command(ctx, args, stdin, stdout, stderr),
            Self::BuiltinType => type_command(ctx, args, stdin, stdout, stderr),
            Self::BuiltinPwd => pwd_command(ctx, args, stdin, stdout, stderr),
            Self::BuiltinCd => cd_command(ctx, args, stdin, stdout, stderr),
            Self::BuiltinComplete => complete_command(ctx, args, stdin, stdout, stderr),
            Self::BuiltinJobs => jobs_command(ctx, args, stdin, stdout, stderr),
            Self::BuiltinHistory => history_command(ctx, args, stdin, stdout, stderr),
            Self::BuiltinDeclare => declare_command(ctx, args, stdin, stdout, stderr),
            Self::External(_) => {}
        }
    }
}

fn exit_command(
    ctx: &mut ShellContext,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut stderr: Box<dyn Write + Send>,
) {
    let _ = writeln!(stdout, "exit");

    let write_history_on_exit = |ctx: &mut ShellContext| {
        if let Ok(history_file_path) = env::var("HISTFILE") {
            ctx.write_history_to_file(&PathBuf::from(history_file_path), true)
        } else {
            Ok(())
        }
    };

    if args.is_empty() {
        match write_history_on_exit(ctx) {
            Ok(_) => process::exit(0),
            Err(e) => {
                let _ = writeln!(stderr, "{}", e);
            }
        }
    } else if args.len() >= 2 {
        let _ = writeln!(stderr, "exit: too many arguments");
    } else {
        let _ = match args[0].parse::<i32>() {
            Ok(ret) => match write_history_on_exit(ctx) {
                Ok(_) => process::exit(ret),
                Err(e) => {
                    let _ = writeln!(stderr, "{}", e);
                }
            },
            Err(_) => {
                let _ = writeln!(stderr, "exit: {}: numeric argument required", args[0]);
            }
        };
    }
}

fn echo_command(
    _ctx: &mut ShellContext,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut _stderr: Box<dyn Write + Send>,
) {
    let _ = writeln!(stdout, "{}", args.join(" "));
}

fn type_command(
    ctx: &mut ShellContext,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut _stderr: Box<dyn Write + Send>,
) {
    for command in args {
        if Command::is_builtin(command) {
            let _ = writeln!(stdout, "{} is a shell builtin", command);
        } else {
            let executables = ctx.external_executables();
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
    ctx: &mut ShellContext,
    _args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut _stderr: Box<dyn Write + Send>,
) {
    let path = ctx.cwd();
    let _ = writeln!(stdout, "{}", path.display());
}

fn cd_command(
    ctx: &mut ShellContext,
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

    let _ = ctx
        .cd(path.clone())
        .map_err(|e| writeln!(stderr, "cd: {}: {}", path.display(), e));
}

fn complete_command(
    ctx: &mut ShellContext,
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
        if print_flag {
            if let Some(completer_path) = ctx.get_completer(&name) {
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
            ctx.unregister_completer(name);
        } else {
            // Or use: `if let Some(ref path) = completer_path`
            // where `ref` indicates: use borrow in pattern matching, instead of move
            if let Some(path) = completer_path.as_ref() {
                ctx.register_completer(name, path.clone());
            }
        }
    }
}

fn jobs_command(
    ctx: &mut ShellContext,
    _args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut _stderr: Box<dyn Write + Send>,
) {
    let jobs = ctx.jobs();
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

fn history_command(
    ctx: &mut ShellContext,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut stderr: Box<dyn Write + Send>,
) {
    let total = ctx.history().len();
    let mut n = total;

    if !args.is_empty() {
        if args[0] == "-r" || args[0] == "-w" || args[0] == "-a" {
            if args.len() < 2 {
                let _ = writeln!(stderr, "history: {}: option requires an argument", args[0]);
                return;
            }
            let file_path = PathBuf::from(args[1]);
            let result = match args[0] {
                "-r" => ctx.read_history_from_file(&file_path),
                "-w" => ctx.write_history_to_file(&file_path, false),
                "-a" => ctx.write_history_to_file(&file_path, true),
                _ => unreachable!(),
            };

            if let Err(e) = result {
                let _ = writeln!(stderr, "history: {}: {}", args[0], e);
            }
            return;
        } else if let Ok(num) = args[0].parse::<usize>() {
            n = num;
        }
    }

    for (i, history) in ctx.history().iter().enumerate().skip(total - n) {
        let _ = writeln!(stdout, "   {}  {}", i + 1, history);
    }
}

fn declare_command(
    ctx: &mut ShellContext,
    args: Vec<&str>,
    mut _stdin: Box<dyn Read + Send>,
    mut stdout: Box<dyn Write + Send>,
    mut stderr: Box<dyn Write + Send>,
) {
    if !args.is_empty() {
        if args[0] == "-p" {
            if args.len() < 2 {
                let _ = writeln!(stderr, "declare: {}: option requires an argument", args[0]);
                return;
            }
            let var_name = args[1].to_string();
            match ctx.get_shell_variable_value(&var_name) {
                Some(var_value) => {
                    let _ = writeln!(stdout, "declare -- {}=\"{}\"", var_name, var_value);
                }
                None => {
                    let _ = writeln!(stderr, "declare: {}: not found", var_name);
                }
            }
            return;
        }

        let variable = args[0].split("=").collect::<Vec<_>>();
        let var_name = variable[0].to_string();
        let var_value = variable[1].to_string();

        if !validate_var_name(&var_name) {
            let _ = writeln!(stderr, "declare: `{}={}': not a valid identifier", var_name, var_value);
            return;
        }

        ctx.declare_shell_variable(var_name, var_value);
    }
}

fn validate_var_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    for (i, char) in name.chars().enumerate() {
        if i == 0 {
            if !char.is_ascii_alphabetic() || char != '_' {
                return false;
            }
        } else {
            if !char.is_ascii_digit() || !char.is_ascii_alphabetic() || char != '_' {
                return false;
            }
        }
    }

    true
}
