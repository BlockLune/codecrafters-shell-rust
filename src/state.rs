use std::env;
use std::path::{Path, PathBuf};

pub struct AppState {
    cwd: PathBuf,
}

impl AppState {
    pub fn default() -> Result<Self, String> {
        let cwd =
            env::current_dir().map_err(|_| String::from("failed to get current directory"))?;
        Ok(Self { cwd })
    }

    pub fn get_cwd(&self) -> &Path {
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
}
