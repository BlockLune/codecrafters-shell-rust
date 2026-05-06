use std::env;
use std::path::PathBuf;

pub struct AppState {
    cwd: Option<PathBuf>,
}

impl AppState {
    pub fn default() -> Self {
        Self {
            cwd: env::current_dir().ok(),
        }
    }

    pub fn get_cwd(&self) -> Option<&PathBuf> {
        self.cwd.as_ref()
    }

    pub fn cd(&mut self, path: PathBuf) -> Result<(), String> {
        if path.starts_with("/") {
            if !path.exists() {
                return Err(String::from("No such file or directory"));
            }
            self.cwd = Some(path);
        } else {
            if self.cwd.is_none() {
                return Err(String::from("No such file or directory"));
            }
            let Ok(canonicalized) = self.cwd.as_ref().unwrap().join(path).canonicalize() else {
                return Err(String::from("No such file or directory"));
            };
            self.cwd = Some(canonicalized);
        }

        Ok(())
    }
}
