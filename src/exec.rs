use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn build_executables() -> HashMap<String, PathBuf> {
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
