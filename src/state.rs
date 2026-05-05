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

    pub fn cd(&mut self, path: PathBuf) {
        if path.starts_with("/") {
            self.cwd = Some(path);
        }
    }
}
