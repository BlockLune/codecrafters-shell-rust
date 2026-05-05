use std::io::{self, Write};

fn main() {
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
