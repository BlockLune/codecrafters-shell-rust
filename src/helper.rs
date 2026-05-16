use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use std::fs;

use crate::command;
use crate::state::AppState;

pub struct ShellHelper {
    commands: Vec<String>,
}

impl ShellHelper {
    pub fn new(app_state: &AppState) -> Self {
        // builtin commands
        let mut commands: Vec<String> = command::BUILTIN_COMMANDS
            .iter()
            .map(|v| v.to_string())
            .collect();

        // external commands
        commands.extend(app_state.external_executables().keys().cloned());

        commands.sort();
        commands.dedup();
        ShellHelper { commands }
    }

    fn complete_command(&self, prefix: &str) -> Vec<Pair> {
        self.commands
            .iter()
            .filter(|command| command.starts_with(prefix))
            .map(|command| Pair {
                display: command.clone(),
                replacement: command.clone(),
            })
            .collect()
    }

    fn complete_path(&self, prefix: &str) -> Vec<Pair> {
        let (dir_path, display_prefix, file_prefix) = match prefix.rfind('/') {
            Some(idx) => (&prefix[..=idx], &prefix[..=idx], &prefix[idx + 1..]),
            None => ("./", "", prefix),
        };

        let Ok(read_dir) = fs::read_dir(dir_path) else {
            return Vec::new();
        };

        read_dir
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let mut name = entry.file_name().into_string().ok()?;
                if !name.starts_with(file_prefix) {
                    return None;
                }

                if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                    name.push('/');
                }

                let full_path = format!("{}{}", display_prefix, name);

                Some(Pair {
                    display: full_path.clone(),
                    replacement: full_path,
                })
            })
            .collect()
    }
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let word_start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let prefix = &line[word_start..pos];

        let mut matches = if word_start == 0 {
            self.complete_command(prefix)
        } else {
            self.complete_path(prefix)
        };

        matches.sort_by(|a, b| a.display.cmp(&b.display));

        if matches.len() == 1 {
            let pair = &mut matches[0];
            if !pair.replacement.ends_with('/') {
                pair.replacement.push(' ');
            }
        }

        Ok((word_start, matches))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Validator for ShellHelper {}

impl Highlighter for ShellHelper {}

impl Helper for ShellHelper {}
