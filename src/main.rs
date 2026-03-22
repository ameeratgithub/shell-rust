use crossterm::cursor;

use crate::{
    lexer::{Lexer, LexerError},
    parser::Parser,
    vm::{VM, VMError},
};
use std::io::{self, Write};

#[allow(unused_imports)]
mod eval;
mod keywords;
mod lexer;
mod parser;
mod vm;

fn main() {
    loop {
        let mut command: String = String::new();

        if let Ok((col, _row)) = cursor::position() {
            if col > 0 {
                println!();
            }
        }

        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut command).unwrap();

        command = command.trim().to_string();

        let tokens = loop {
            let mut lexer = Lexer::new(&command);
            match lexer.scan_tokens() {
                Ok(tokens) => break tokens,
                Err(LexerError::UnterminatedString) => {
                    print!("> ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut command).unwrap();
                }
                Err(_) => {
                    eprintln!("Lexer Error");
                    return;
                }
            }
        };
        // println!("{tokens:?}");

        let parse_result = Parser::parse(tokens);
        match parse_result {
            Ok(ast) => {
                let vm_result = VM::execute(ast);
                match vm_result {
                    Err(VMError::Exit) => {
                        break;
                    }
                    Err(VMError::Other(e)) => {
                        eprintln!("{e}");
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("{e:?}"),
        }
    }
    // eval::run();
}
