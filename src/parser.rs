use std::fs::File;

pub struct ParsedInput<'a> {
    pub commands: Vec<ParsedCommand<'a>>,
    pub run_in_background: bool,
}

pub struct ParsedCommand<'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
    pub stdout_redirect: Option<File>,
    pub stderr_redirect: Option<File>,
}

pub fn parse_input<'a>(tokens: &'a [String]) -> Result<ParsedInput<'a>, String> {
    let (tokens, run_in_background) = strip_background_flag(tokens);
    let commands: Vec<ParsedCommand<'a>> = split_pipeline(tokens)
        .iter()
        .map(|&command_tokens| parse_command(command_tokens))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ParsedInput {
        commands,
        run_in_background,
    })
}

fn strip_background_flag(tokens: &[String]) -> (&[String], bool) {
    if tokens.is_empty() {
        return (&[], false);
    }

    let run_in_background = tokens.last().map_or(false, |token| token == "&");

    if run_in_background {
        (&tokens[0..tokens.len() - 1], run_in_background)
    } else {
        (&tokens, run_in_background)
    }
}

fn split_pipeline<'a>(tokens: &'a [String]) -> Vec<&'a [String]> {
    let mut commands: Vec<&[String]> = Vec::new();
    let mut start = 0;
    for (i, token) in tokens.iter().enumerate() {
        if token == "|" && start != i {
            commands.push(&tokens[start..i]);
            start = i + 1;
        }
    }
    commands.push(&tokens[start..]);
    commands
}

fn parse_command<'a>(tokens: &'a [String]) -> Result<ParsedCommand<'a>, String> {
    let command: &str = tokens.first().unwrap().as_str();
    let mut args: Vec<&str> = tokens[1..tokens.len()].iter().map(|s| s.as_str()).collect();
    let mut stdout_redirect = None;
    let mut stderr_redirect = None;

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
                ">" | "1>" => stdout_redirect = Some(create_fd(filepath, false)?),
                "2>" => stderr_redirect = Some(create_fd(filepath, false)?),
                ">>" | "1>>" => stdout_redirect = Some(create_fd(filepath, true)?),
                "2>>" => stderr_redirect = Some(create_fd(filepath, true)?),
                _ => unreachable!(),
            }

            args.drain(i..=i + 1);
        } else {
            i += 1;
        }
    }

    Ok(ParsedCommand {
        name: command,
        args,
        stdout_redirect,
        stderr_redirect,
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
