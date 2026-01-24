use std::{
    env, fs,
    io::{self, Write},
    path::{self, Path, PathBuf},
    process::{self, Command},
    str::FromStr,
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
            "pwd" => {
                if let Ok(curr_dir) = env::current_dir() {
                    println!("{}", curr_dir.display());
                }
            }
            "cd" => {
                let Ok(path) = PathBuf::from_str(&args[1]);
                if path.is_dir() {
                    env::set_current_dir(path).unwrap();
                } else {
                    println!("cd: {}: No such file or directory", path.display())
                }
            }
            "type" => {
                if KEYWORDS.contains(args[1]) {
                    println!("{} is a shell builtin", args[1])
                } else {
                    let file_name = args[1];

                    if let Some(path) = check_executable_file_exists_in_paths(file_name) {
                        println!("{} is {}", file_name, path);
                    } else {
                        println!("{}: not found", file_name);
                    }
                }
            }
            _ => {
                let executable_path = check_executable_file_exists_in_paths(args[0]);
                if let Some(_) = executable_path {
                    Command::new(args[0]).args(&mut args[1..]).status().unwrap();
                } else {
                    println!("{command}: command not found");
                }
            }
        }
    }
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
