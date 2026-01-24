#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();
        if command == "exit" {
            break;
        }

        let args = command.split_whitespace().collect::<Vec<&str>>();
        let command = args[0];
        if command == "echo" {
            let output = &args[1..].join(" ");
            println!("{output}");
        } else {
            println!("{command}: command not found");
        }
    }
}
