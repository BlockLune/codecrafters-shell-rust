use anyhow::{Context, Result, bail};
use std::fs::File;

pub struct ParsedInput {
    pub commands: Vec<ParsedCommand>,
    pub run_in_background: bool,
}

pub struct ParsedCommand {
    pub name: String,
    pub args: Vec<String>,
    pub stdout_redirect: Option<File>,
    pub stderr_redirect: Option<File>,
}

pub fn parse_input(tokens: &[String]) -> Result<ParsedInput> {
    let (tokens, run_in_background) = strip_background_flag(tokens);
    let commands: Vec<ParsedCommand> = split_pipeline(tokens)
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

    let run_in_background = tokens.last().is_some_and(|token| token == "&");

    if run_in_background {
        (&tokens[0..tokens.len() - 1], run_in_background)
    } else {
        (tokens, run_in_background)
    }
}

fn split_pipeline(tokens: &[String]) -> Vec<&[String]> {
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

fn parse_command(tokens: &[String]) -> Result<ParsedCommand> {
    let command: String = tokens.first().unwrap().to_string();
    let mut args: Vec<String> = tokens[1..tokens.len()]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut stdout_redirect = None;
    let mut stderr_redirect = None;

    let mut i = 0;
    while i < args.len() {
        let token = &args[i];
        if token == ">"
            || token == "1>"
            || token == "2>"
            || token == ">>"
            || token == "1>>"
            || token == "2>>"
        {
            if i + 1 >= args.len() {
                bail!("no redirection target for `{}`", token);
            }
            let filepath = &args[i + 1];
            match token.as_str() {
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

fn create_fd(filepath: &str, appending: bool) -> Result<File> {
    if appending {
        File::options().append(true).create(true).open(filepath)
    } else {
        File::create(filepath)
    }
    .with_context(|| format!("failed to create file: {}", filepath))
}
