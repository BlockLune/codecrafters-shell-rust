use std::process;

use crate::state::AppState;

pub fn exec_external(app_state: &AppState, command: &str, args: Vec<&str>) {
    if !app_state.get_external_executables().contains_key(command) {
        println!("{}: command not found", command);
        return;
    }

    let output = process::Command::new(command)
        .current_dir(app_state.get_cwd())
        .args(args)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{}", stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }
}
