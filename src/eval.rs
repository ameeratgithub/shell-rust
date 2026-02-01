use std::{
    env, fs,
    io::{self, Write},
    iter::Peekable,
    path::{self, Path, PathBuf},
    process::{self, Command},
    str::{Chars, FromStr},
};

use crate::keywords::KEYWORDS;
use std::os::unix::fs::PermissionsExt;

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

        match command {
            "echo" => {
                let output = args
                    .iter()
                    .skip(1)
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");

                println!("{output}");
            }
            "exit" => {
                break;
            }
            "pwd" => {
                if let Ok(curr_dir) = env::current_dir() {
                    println!("{}", curr_dir.display());
                }
            }
            "cd" => {
                if args[1] == "~"
                    && let Ok(home_path) = env::var("HOME")
                {
                    env::set_current_dir(home_path).unwrap();
                    continue;
                }

                let Ok(path) = PathBuf::from_str(&args[1]);
                if path.is_dir() {
                    env::set_current_dir(path).unwrap();
                } else {
                    println!("cd: {}: No such file or directory", path.display())
                }
            }
            "type" => {
                let first_arg = args[1].as_str();
                if KEYWORDS.contains(first_arg) {
                    println!("{} is a shell builtin", first_arg)
                } else {
                    if let Some(path) = check_executable_file_exists_in_paths(first_arg) {
                        println!("{} is {}", first_arg, path);
                    } else {
                        println!("{}: not found", first_arg);
                    }
                }
            }
            _ => {
                // let executable_path = check_executable_file_exists_in_paths(command);
                let execution_result = Command::new(command).args(&mut args[1..]).status();
                if let Err(_) = execution_result {
                    let command = args[0].as_str();
                    println!("{command}: command not found");
                }
            }
        }
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

fn check_executable_file_exists_in_paths(file: &str) -> Option<String> {
    if let Ok(paths) = env::var("PATH") {
        let directories = env::split_paths(&paths);
        for directory in directories {
            let path = directory.join(file);
            println!("path:{path:?}");
            if path.exists() {
                let metadata = fs::metadata(&path).unwrap();
                let mode = metadata.permissions().mode();
                if mode & 0o111 != 0 {
                    return path.to_str().map(|str| str.to_owned());
                }
            }
        }
    }
    None
}
