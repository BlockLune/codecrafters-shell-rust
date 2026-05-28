use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process;

use rustyline::history::History;

use crate::job::Job;
use crate::context::ShellContext;

pub const BUILTIN_COMMANDS: &[&str] = &[
    "exit", "echo", "type", "pwd", "cd", "complete", "jobs", "history",
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
            Self::External(_) => {}
        }
    }
}

fn exit_command(
    _ctx: &mut ShellContext,
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
    let total = ctx.editor().history().len();
    let mut n = total;

    if !args.is_empty() {
        if args[0] == "-r" {
            if args.len() < 2 {
                let _ = writeln!(stderr, "history: -r: option requires an argument");
                return;
            }
            let history_file_path = PathBuf::from(args[1]);
            if let Err(_) = ctx
                .editor_mut()
                .history_mut()
                .load(&history_file_path)
            {
                let _ = writeln!(
                    stderr,
                    "history: -r: failed to load {}",
                    history_file_path.display()
                );
            }
            return;
        } else if args[0] == "-w" {
            if args.len() < 2 {
                let _ = writeln!(stderr, "history: -w: option requires an argument");
                return;
            }
            let history_file_path = PathBuf::from(args[1]);
            if let Err(_) = ctx
                .editor_mut()
                .history_mut()
                .save(&history_file_path)
            {
                let _ = writeln!(
                    stderr,
                    "history: -w: failed to load {}",
                    history_file_path.display()
                );
            }
            return;
        } else if let Ok(num) = args[0].parse::<usize>() {
            n = num;
        }
    }

    for (i, history) in ctx
        .editor()
        .history()
        .iter()
        .enumerate()
        .skip(total - n)
    {
        let _ = writeln!(stdout, "   {}  {}", i + 1, history);
    }
}
