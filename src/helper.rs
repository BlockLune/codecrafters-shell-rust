use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;

use crate::command;
use crate::state::AppState;

pub struct ShellHelper {
    commands: Vec<String>,
    app_state: Rc<RefCell<AppState>>,
}

impl ShellHelper {
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        // builtin commands
        let mut commands: Vec<String> = command::BUILTIN_COMMANDS
            .iter()
            .map(|v| v.to_string())
            .collect();

        // external commands
        commands.extend(app_state.borrow().external_executables().keys().cloned());

        commands.sort();
        commands.dedup();
        ShellHelper {
            commands,
            app_state,
        }
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

    fn complete_with_completer(
        &self,
        completer: &PathBuf,
        args: Vec<&str>,
        comp_line: &str,
        comp_point: usize,
    ) -> Vec<Pair> {
        let mut ret = Vec::new();
        let app_state = self.app_state.borrow();

        let Ok(output) = process::Command::new(completer)
            .current_dir(app_state.cwd())
            .args(args)
            .env("COMP_LINE", comp_line)
            .env("COMP_POINT", comp_point.to_string())
            .output()
        else {
            return ret;
        };

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        if !stdout_str.is_empty() {
            stdout_str.lines().for_each(|line| {
                ret.push(Pair {
                    display: line.to_string(),
                    replacement: line.to_string(),
                });
            });
        }

        ret
    }
}

fn first_char_pos(line: &str) -> usize {
    let line: Vec<_> = line.chars().collect();

    let mut i = 0;
    while i < line.len() {
        if line[i] != ' ' {
            break;
        }
        i += 1;
    }
    i
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

        let mut matches = if word_start == first_char_pos(line) {
            self.complete_command(prefix)
        } else {
            let parts: Vec<_> = line[..word_start].split_whitespace().collect();

            let command = parts.first().copied().unwrap();
            let preceding_word = if parts.len() >= 2 {
                parts.last().copied().unwrap()
            } else {
                ""
            };

            let args = vec![command, prefix, preceding_word];

            match self.app_state.borrow().get_completer(command) {
                Some(completer) => self.complete_with_completer(completer, args, line, pos),
                None => self.complete_path(prefix),
            }
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
