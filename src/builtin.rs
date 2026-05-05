use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

pub fn exit_command() {
    process::exit(0);
}

pub fn echo_command(s: &str) {
    println!("{}", s);
}

pub fn type_command(command: Option<&str>) {
    if command.is_none() {
        return;
    }
    let command = command.unwrap();

    if command == "exit" || command == "echo" || command == "type" {
        println!("{} is a shell builtin", command);
    } else {
        let executables = build_executables();
        if executables.contains_key(command) {
            println!(
                "{} is {}",
                command,
                executables
                    .get(command)
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            )
        } else {
            println!("{}: not found", command);
        }
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
            if !executables.contains_key(&executable_name) {
                executables.insert(executable_name, entry.path());
            }
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
