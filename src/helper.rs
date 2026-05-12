use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use std::fs;

use crate::state::AppState;

pub struct ShellHelper {
    commands: Vec<String>,
}

impl ShellHelper {
    pub fn new(app_state: &AppState) -> Self {
        // builtin commands
        let mut commands: Vec<String> = vec!["exit", "echo", "type", "pwd", "cd"]
            .into_iter()
            .map(String::from)
            .collect();

        // external commands
        commands.extend(app_state.get_external_executables().keys().cloned());

        commands.sort();
        commands.dedup();
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
        let matches = if word_start == 0 {
            // command
            let cmd_matches: Vec<_> = self
                .commands
                .iter()
                .filter(|command| command.starts_with(prefix))
                .collect();

            let single = cmd_matches.len() == 1;
            cmd_matches
                .iter()
                .map(|command| Pair {
                    display: command.to_string(),
                    replacement: if single {
                        format!("{} ", command)
                    } else {
                        command.to_string()
                    },
                })
                .collect()
        } else {
            // path
            let mut directory_path = "./";
            let mut file_prefix = prefix;
            if prefix.contains('/') {
                let directory_path_end = prefix.rfind('/').unwrap();
                directory_path = &prefix[..=directory_path_end];
                file_prefix = &prefix[directory_path_end + 1..];
            }

            let dir_name = if directory_path == "./" {
                ""
            } else {
                directory_path
            };

            let mut dir_entries: Vec<_> = fs::read_dir(directory_path)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|dir_entry| {
                    dir_entry
                        .file_name()
                        .to_string_lossy()
                        .to_string()
                        .starts_with(file_prefix)
                })
                .collect();

            dir_entries.sort_by(|a, b| {
                a.file_name()
                    .to_string_lossy()
                    .to_string()
                    .cmp(&b.file_name().to_string_lossy().to_string())
            });

            let single = dir_entries.len() == 1;
            dir_entries
                .iter()
                .map(|entry| {
                    let is_dir = entry.file_type().unwrap().is_dir();
                    let name = entry.file_name().to_string_lossy().to_string();
                    Pair {
                        display: format!("{}{}{}", dir_name, name, if is_dir { "/" } else { "" }),
                        replacement: format!(
                            "{}{}{}",
                            dir_name,
                            name,
                            if single {
                                if is_dir { "/" } else { " " }
                            } else {
                                ""
                            }
                        ),
                    }
                })
                .collect()
        };

        Ok((word_start, matches))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Validator for ShellHelper {}

impl Highlighter for ShellHelper {}

impl Helper for ShellHelper {}
