use crate::lexer::Lexer;
use std::io::{self, Write};

#[allow(unused_imports)]
mod eval;
mod keywords;
mod lexer;

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
        let lexer = Lexer::new();
        let tokens = lexer.scan(command);
        if let Err(e) = tokens {
            eprintln!("Lexer Error: {e:?}");
            return;
        }

        let tokens = tokens.unwrap();
        
    }
    // eval::run();
}
