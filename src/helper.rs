use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use crate::state::AppState;

pub struct ShellHelper {
    commands: Vec<String>,
}

impl ShellHelper {
    pub fn new(app_state: &AppState) -> Self {
        let mut commands: Vec<String> = vec!["exit", "echo", "type", "pwd", "cd"]
            .into_iter()
            .map(String::from)
            .collect();
        commands.extend(app_state.get_external_executables().keys().cloned());
        ShellHelper { commands }
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
            .commands
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
