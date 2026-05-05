use std::io::{self, Write};

mod builtin;
mod exec;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let commands: Vec<_> = command.split_whitespace().collect();

        match commands.first() {
            Some(&"exit") => builtin::exit_command(),
            Some(&"echo") => builtin::echo_command(&commands[1..].join(" ")),
            Some(&"type") => builtin::type_command(commands.get(1).copied()),
            Some(command) => exec::exec_external(command, &commands[1..]),
            None => (),
        }
    }
}
