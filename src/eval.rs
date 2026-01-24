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

        let args = parse_command_with_args(command);
        let command = args[0].as_str();

        match command {
            "echo" => {
                let first_arg = args[1].as_str();
                println!("{first_arg}");
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
                let command = args[0].as_str();
                let executable_path = check_executable_file_exists_in_paths(command);
                if let Some(_) = executable_path {
                    let mut parsed_args: Vec<String> = args[1..]
                        .iter()
                        .map(|arg| {
                            if arg.chars().nth(0) == Some('\'') {
                                parse_string(&mut arg.chars().peekable()).unwrap()
                            } else {
                                arg.to_string()
                            }
                        })
                        .collect();

                    Command::new(command)
                        .args(&mut parsed_args)
                        .status()
                        .unwrap();
                } else {
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
        if *c == '\'' {
            args.push(parse_string(chars).unwrap());
        } else {
            args.push(parse_command(chars));
        }
    }
    args
}

fn parse_command(command: &mut Peekable<Chars>) -> String {
    let mut str = String::new();
    while let Some(c) = command.next() {
        if c.is_whitespace() {
            break;
        }
        str.push(c);
    }
    str
}

fn parse_string(arg: &mut Peekable<Chars>) -> Option<String> {
    // Advancing because first "'" has already been checked
    arg.next();

    let mut str = String::new();
    while let Some(c) = arg.next() {
        if c == '\'' {
            return Some(str);
        }
        str.push(c);
    }

    None
}

fn check_executable_file_exists_in_paths(file: &str) -> Option<String> {
    if let Ok(paths) = env::var("PATH") {
        let directories = env::split_paths(&paths);
        for directory in directories {
            let path = directory.join(file);
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
