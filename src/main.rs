use crate::{
    lexer::Lexer,
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
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();

        if command == "exit" {
            break;
        }

        let lexer = Lexer::new();
        let tokens = lexer.scan_until_complete(command);
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
