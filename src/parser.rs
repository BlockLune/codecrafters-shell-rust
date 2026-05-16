use std::fs::File;
use std::io;

pub struct ParsedCommand<'a> {
    pub command: &'a str,
    pub args: Vec<&'a str>,
    pub stdout: Box<dyn io::Write>,
    pub stderr: Box<dyn io::Write>,
    pub run_in_background: bool,
}

pub fn parse_command(tokens: &[String]) -> Result<ParsedCommand<'_>, String> {
    let command: &str = tokens.first().unwrap().as_str();
    let run_in_background = tokens.last().map_or(false, |token| token == "&");

    let end_pos = if run_in_background {
        tokens.len() - 1
    } else {
        tokens.len()
    };

    let mut args: Vec<&str> = tokens[1..end_pos].iter().map(|s| s.as_str()).collect();
    let mut stdout: Box<dyn io::Write> = Box::new(io::stdout());
    let mut stderr: Box<dyn io::Write> = Box::new(io::stderr());

    let mut i = 0;
    while i < args.len() {
        let token = args[i];
        if token == ">"
            || token == "1>"
            || token == "2>"
            || token == ">>"
            || token == "1>>"
            || token == "2>>"
        {
            if i + 1 >= args.len() {
                return Err(String::from("no redirection target"));
            }
            let filepath = args[i + 1];
            match token {
                ">" | "1>" => stdout = Box::new(create_fd(filepath, false)?),
                "2>" => stderr = Box::new(create_fd(filepath, false)?),
                ">>" | "1>>" => stdout = Box::new(create_fd(filepath, true)?),
                "2>>" => stderr = Box::new(create_fd(filepath, true)?),
                _ => unreachable!(),
            }

            args.drain(i..=i + 1);
        } else {
            i += 1;
        }
    }

    Ok(ParsedCommand {
        command,
        args,
        stdout,
        stderr,
        run_in_background,
    })
}

fn create_fd(filepath: &str, appending: bool) -> Result<File, String> {
    let file = if appending {
        File::options().append(true).create(true).open(filepath)
    } else {
        File::create(filepath)
    };

    file.map_err(|_| format!("failed to create file: {}", filepath))
}
