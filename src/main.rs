use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};

fn main() {
    let mut executables = HashMap::new();

    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let executeble_name = entry.file_name().to_string_lossy().to_string();
                    executables.insert(executeble_name, entry.path());
                }
            }
        }
    }

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let commands: Vec<_> = command.split_whitespace().collect();

        match commands.first() {
            Some(&"exit") => {
                break;
            }
            Some(&"echo") => {
                println!("{}", commands[1..].join(" "));
            }
            Some(&"type") => {
                if commands.len() > 1 {
                    if commands[1] == "exit" || commands[1] == "echo" || commands[1] == "type" {
                        println!("{} is a shell builtin", commands[1]);
                    } else if executables.contains_key(commands[1]) {
                        println!(
                            "{} is {}",
                            commands[1],
                            executables
                                .get(commands[1])
                                .unwrap()
                                .to_string_lossy()
                                .to_string()
                        )
                    } else {
                        println!("{}: not found", commands[1]);
                    }
                }
            }
            Some(_) => {
                println!("{}: command not found", commands[0]);
            }
            None => (),
        }
    }
}
