use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::job::Job;

pub struct AppState {
    cwd: PathBuf,
    external_executables: HashMap<String, PathBuf>,
    completers: HashMap<String, PathBuf>,
    background_jobs: Vec<Job>,
}

impl AppState {
    pub fn default() -> Result<Self, String> {
        let cwd =
            env::current_dir().map_err(|_| String::from("failed to get current directory"))?;
        let external_executables = build_executables();
        let completers = HashMap::new();
        let background_jobs = Vec::new();

        Ok(Self {
            cwd,
            external_executables,
            completers,
            background_jobs,
        })
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn external_executables(&self) -> &HashMap<String, PathBuf> {
        &self.external_executables
    }

    pub fn register_completion(&mut self, name: String, completer_path: PathBuf) {
        let _ = &self.completers.insert(name, completer_path);
    }

    pub fn get_completer(&self, name: &str) -> Option<&PathBuf> {
        self.completers.get(name)
    }

    pub fn unregister_completion(&mut self, name: String) {
        let _ = self.completers.remove(&name);
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

    pub fn add_background_job(&mut self, command_line: &str, pid: u32) -> usize {
        self.background_jobs
            .push(Job::new(self.background_jobs.len() + 1, command_line, pid));
        self.background_jobs.len()
    }

    pub fn jobs(&self) -> &Vec<Job> {
        &self.background_jobs
    }
}

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
