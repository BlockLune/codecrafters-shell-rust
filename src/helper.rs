use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use std::fs;

use crate::state::AppState;

pub struct ShellHelper {
    candidates: Vec<String>,
}

impl ShellHelper {
    pub fn new(app_state: &AppState) -> Self {
        // builtin commands
        let mut candidates: Vec<String> = vec!["exit", "echo", "type", "pwd", "cd"]
            .into_iter()
            .map(String::from)
            .collect();

        // external commands
        candidates.extend(app_state.get_external_executables().keys().cloned());

        candidates.sort();
        ShellHelper { candidates }
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

        let mut matches: Vec<_> = self
            .candidates
            .iter()
            .filter(|command| command.starts_with(prefix))
            .map(|command| Pair {
                display: command.to_string(),
                replacement: format!("{} ", command),
            })
            .collect();

        // path
        let mut directory_path = "./";
        let mut file_prefix = prefix;
        if prefix.contains('/') {
            let directory_path_end = prefix.rfind('/').unwrap();
            directory_path = &prefix[..=directory_path_end];
            file_prefix = &prefix[directory_path_end + 1..];
        }
        matches.extend(
            fs::read_dir(directory_path)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .map(|dir_entry| dir_entry.file_name().to_string_lossy().to_string())
                .filter(|entry| entry.starts_with(file_prefix))
                .map(|entry| {
                    let display = if directory_path == "./" {
                        format!("{}", entry)
                    } else {
                        format!("{}{}", directory_path, entry)
                    };
                    Pair {
                        display: display.to_string(),
                        replacement: format!("{} ", display),
                    }
                }),
        );

        Ok((word_start, matches))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Validator for ShellHelper {}

impl Highlighter for ShellHelper {}

impl Helper for ShellHelper {}
