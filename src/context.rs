use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;

use rustyline::history::{DefaultHistory, FileHistory, History};
use rustyline::{Editor, error::ReadlineError};

use crate::executor;
use crate::helper::ShellHelper;
use crate::job::Job;
use crate::parser;
use crate::tokenizer;

pub struct ShellContext {
    cwd: PathBuf,
    external_executables: HashMap<String, PathBuf>,
    completers: HashMap<String, PathBuf>,
    background_jobs: Vec<Job>,
    editor: Editor<ShellHelper, DefaultHistory>,
    // tracks how far we've persisted so `history -a` only appends new entries, not the whole file
    history_write_offset: usize,
    shell_variables: HashMap<String, String>,
}

impl ShellContext {
    pub fn new() -> Result<Self, String> {
        let cwd =
            env::current_dir().map_err(|_| String::from("failed to get current directory"))?;
        let external_executables = build_executables();
        let completers = HashMap::new();
        let background_jobs = Vec::new();
        let shell_variables = HashMap::new();

        let mut editor = Editor::with_config(
            rustyline::config::Config::builder()
                .completion_type(rustyline::CompletionType::List)
                .history_ignore_dups(false)
                .map_err(|e| e.to_string())?
                .build(),
        )
        .map_err(|e| e.to_string())?;

        let helper = ShellHelper::new(external_executables.keys().cloned(), &cwd, &completers);
        editor.set_helper(Some(helper));

        let mut ctx = Self {
            cwd,
            external_executables,
            completers,
            background_jobs,
            editor,
            history_write_offset: 0,
            shell_variables,
        };

        if let Ok(history_file_path) = env::var("HISTFILE") {
            ctx.read_history_from_file(&PathBuf::from(history_file_path))
                .map_err(|e| e.to_string())?;
            ctx.history_write_offset = ctx.editor.history().len();
        }

        Ok(ctx)
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn cd(&mut self, path: PathBuf) -> Result<(), String> {
        let target = if path.is_absolute() {
            path
        } else {
            self.cwd.join(path)
        };

        let canonicalized = target
            .canonicalize()
            .map_err(|_| String::from("No such file or directory"))?;

        if !canonicalized.is_dir() {
            return Err(String::from("Not a directory"));
        }

        self.cwd = canonicalized;

        Ok(())
    }

    pub fn external_executables(&self) -> &HashMap<String, PathBuf> {
        &self.external_executables
    }

    pub fn register_completer(&mut self, name: String, completer_path: PathBuf) {
        let _ = &self.completers.insert(name, completer_path);
    }

    pub fn unregister_completer(&mut self, name: String) {
        let _ = self.completers.remove(&name);
    }

    pub fn get_completer(&self, name: &str) -> Option<&PathBuf> {
        self.completers.get(name)
    }

    pub fn jobs(&mut self) -> &mut Vec<Job> {
        &mut self.background_jobs
    }

    pub fn add_background_job(&mut self, command_line: &str, child: process::Child) -> usize {
        let num = self.next_job_number();
        self.background_jobs
            .push(Job::new(num, command_line, child));
        num
    }

    fn next_job_number(&self) -> usize {
        let used: HashSet<usize> = self
            .background_jobs
            .iter()
            .map(|job| job.job_number)
            .collect();
        (1..).find(|n| !used.contains(n)).unwrap()
    }

    pub fn reap_done_jobs(&mut self) {
        let statuses = Job::compute_job_status(&mut self.background_jobs);

        for (job, entry) in self.background_jobs.iter_mut().zip(statuses.iter()) {
            let Some((indicator, status)) = entry else {
                continue;
            };
            if status == "Done" {
                job.done = true;
                println!("{}", job.display(indicator, status));
            }
        }

        self.background_jobs.retain(|job| !job.done);
    }

    pub fn history(&self) -> &FileHistory {
        self.editor.history()
    }

    pub fn read_history_from_file(&mut self, path: &Path) -> Result<(), String> {
        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.is_empty() {
                    self.editor
                        .add_history_entry(&line)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    pub fn write_history_to_file(&mut self, path: &Path, append: bool) -> Result<(), String> {
        let offset = if append { self.history_write_offset } else { 0 };

        let file = if append {
            File::options()
                .append(true)
                .create(true)
                .open(path)
                .map_err(|e| e.to_string())?
        } else {
            File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .map_err(|e| e.to_string())?
        };
        let mut writer = BufWriter::new(file);
        for entry in self.editor.history().iter().skip(offset) {
            let _ = writer.write_all(entry.as_bytes());
            let _ = writer.write_all(b"\n");
        }

        if append {
            self.history_write_offset = self.editor.history().len();
        }

        Ok(())
    }

    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.one_turn() {
                eprintln!("Error: {}", e);
            }
        }
    }

    fn one_turn(&mut self) -> Result<(), String> {
        self.reap_done_jobs();
        self.sync_helper();

        match self.editor.readline("$ ") {
            Ok(line) => self.eval_line(line),
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                Ok(())
            }
            Err(ReadlineError::Eof) => {
                println!(r#"Use "exit" to leave the shell."#);
                Ok(())
            }
            Err(e) => Err(format!("readline error: {}", e)),
        }
    }

    fn eval_line(&mut self, line: String) -> Result<(), String> {
        let _ = self.editor.add_history_entry(line.as_str());

        let tokens = tokenizer::tokenize(&line)?;
        let mut parsed_input = parser::parse_input(&tokens)?;

        for command in &mut parsed_input.commands {
            for arg in &mut command.args {
                let mut result = String::with_capacity(arg.len());
                let mut chars = arg.chars().peekable();

                while let Some(ch) = chars.next() {
                    if ch != '$' {
                        result.push(ch);
                        continue;
                    }

                    if chars.peek() == Some(&'{') {
                        chars.next();
                        let mut var_name = String::new();
                        loop {
                            match chars.next() {
                                Some('}') => {
                                    break;
                                }
                                Some(ch) => {
                                    var_name.push(ch);
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                        if let Some(var_value) = self.get_shell_variable_value(&var_name) {
                            result.push_str(var_value);
                        }
                        continue;
                    }

                    let mut var_name = String::new();
                    while let Some(ch) = chars.next() {
                        if ch.is_whitespace() {
                            break;
                        }
                        var_name.push(ch);
                    }
                    if var_name.is_empty() {
                        result.push('$');
                    } else if let Some(value) = self.get_shell_variable_value(&var_name) {
                        result.push_str(value);
                    }
                }

                *arg = result;
            }
            command.args.retain(|a| !a.is_empty());
        }

        executor::execute(self, parsed_input)?;

        Ok(())
    }

    // rustyline owns the Helper, but completions depend on mutable shell state (cwd, completers).
    // we can't give rustyline a reference into ShellContext, so push a snapshot before each prompt.
    fn sync_helper(&mut self) {
        if let Some(helper) = self.editor.helper_mut() {
            helper.sync_from_context(&self.cwd, &self.completers);
        }
    }

    pub fn declare_shell_variable(&mut self, name: String, value: String) {
        let _ = self.shell_variables.insert(name, value);
    }

    pub fn get_shell_variable_value(&self, name: &str) -> Option<&String> {
        self.shell_variables.get(name)
    }
}

// scanned once at startup -- newly installed binaries won't be found until shell restart.
fn build_executables() -> HashMap<String, PathBuf> {
    let mut executables = HashMap::new();

    let paths = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => return executables,
    };

    for entry in env::split_paths(&paths)
        .filter_map(|path| fs::read_dir(path).ok())
        .flatten()
        .filter_map(|entry| entry.ok())
    {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        if metadata.is_file() && is_executable(&entry.path()) {
            let executable_name = entry.file_name().to_string_lossy().to_string();
            // entry().or_insert() gives first-match-wins, matching POSIX PATH precedence.
            executables.entry(executable_name).or_insert(entry.path());
        }
    }

    executables
}

#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
