use std::fs::Metadata;
use std::{
    env, fs,
    io::{self, Write},
    iter::Peekable,
    path::{self, Path, PathBuf},
    process::{self, Command},
    str::{Chars, FromStr},
};

use crate::keywords::KEYWORDS;

pub fn run() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        let command = command.trim();
        if command == "exit" {
            break;
        }

        let mut args = parse_command_with_args(command);
        let command = args[0].as_str();

        // println!("args:{args:?}");

        // match command {
            
            
           
        // }
    }
}

fn parse_command_with_args(command: &str) -> Vec<String> {
    let mut args = vec![];
    let chars = &mut command.chars().peekable();
    while let Some(c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            args.push(parse_args(chars));
        }
    }
    args
}

fn parse_args(chars: &mut Peekable<Chars>) -> String {
    let mut str = String::new();
    while let Some(c) = chars.peek() {
        if *c == '\\' {
            chars.next();
            if let Some(c2) = chars.next() {
                str.push(c2);
            }
        } else if c.is_whitespace() {
            break;
        } else if *c == '\'' || *c == '"' {
            let c = c.clone();
            str.push_str(&parse_strings(chars, c));
        } else {
            str.push_str(&parse_command(chars));
        }
    }
    str
}

fn parse_command(command: &mut Peekable<Chars>) -> String {
    let mut str = String::new();
    while let Some(c) = command.peek() {
        if c.is_whitespace() {
            break;
        }
        if *c == '\\' {
            command.next();
            if let Some(c) = command.next() {
                str.push(c);
            }
        } else if *c == '\'' || *c == '"' {
            let c = c.to_owned();
            str.push_str(&parse_strings(command, c));
        } else {
            str.push(command.next().unwrap());
        }
    }

    str.trim().to_owned()
}

fn parse_strings(arg: &mut Peekable<Chars>, quote_char: char) -> String {
    let mut str = String::new();

    while let Some(c) = arg.peek() {
        if *c == quote_char {
            str.push_str(&parse_string(arg, quote_char));
        } else if c.is_whitespace() {
            break;
        } else {
            str.push_str(&parse_command(arg));
        }
    }

    str
}

fn parse_string(arg: &mut Peekable<Chars>, quote_char: char) -> String {
    // Advancing because first `quote_char` has already been checked
    arg.next();

    let mut str = String::new();
    while let Some(c) = arg.next() {
        if quote_char == '"' && c == '\\' {
            if let Some(c) = arg.next() {
                if c != quote_char && c != '\\' {
                    str.push('\\');
                }
                str.push(c);
            }
            continue;
        }
        if c == quote_char {
            return str;
        }
        str.push(c);
    }

    str
}

