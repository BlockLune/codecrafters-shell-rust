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

        // direct path
        let file_entries: Vec<_> = fs::read_dir(app_state.get_cwd())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .collect();
        candidates.extend(
            file_entries
                .iter()
                .map(|dir_entry| dir_entry.file_name().to_string_lossy().to_string()),
        );

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

        let matches = self
            .candidates
            .iter()
            .filter(|command| command.starts_with(prefix))
            .map(|command| Pair {
                display: command.to_string(),
                replacement: format!("{} ", command),
            })
            .collect();

        Ok((word_start, matches))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Validator for ShellHelper {}

impl Highlighter for ShellHelper {}

impl Helper for ShellHelper {}
