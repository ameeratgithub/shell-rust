use std::{
    env, fs,
    io::{self, Write},
    process::{self, Command},
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

        let mut args = command.split_whitespace().collect::<Vec<&str>>();
        let command = args[0];

        match command {
            "echo" => {
                let output = &args[1..].join(" ");
                println!("{output}");
            }
            "exit" => {
                break;
            }
            "type" => {
                if KEYWORDS.contains(args[1]) {
                    println!("{} is a shell builtin", args[1])
                } else {
                    check_executable_file_exists_in_paths(&args);
                }
            }
            _ => {
                let executable_path = check_executable_file_exists_in_paths(&args);
                if let Some(_) = executable_path {
                    Command::new(args[0]).args(&mut args[1..]).status().unwrap();
                } else {
                    println!("{command}: command not found");
                }
            }
        }
    }
}

fn check_executable_file_exists_in_paths(args: &Vec<&str>) -> Option<String> {
    if let Ok(paths) = env::var("PATH") {
        let directories = env::split_paths(&paths);
        for directory in directories {
            let path = directory.join(args[1]);
            if path.exists() {
                let metadata = fs::metadata(&path).unwrap();
                let mode = metadata.permissions().mode();
                if mode & 0o111 != 0 {
                    println!("{} is {}", args[1], &path.display());
                    return path.to_str().map(|str| str.to_owned());
                }
            }
        }
        println!("{}: not found", args[1]);
    } else {
        println!("{}: not found", args[1]);
    }

    None
}
