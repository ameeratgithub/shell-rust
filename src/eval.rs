use std::{
    env, fs,
    io::{self, Write},
    process,
};

use crate::keywords::KEYWORDS;
#[cfg(unix)]
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

        let args = command.split_whitespace().collect::<Vec<&str>>();
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
                    if let Ok(path) = env::var("PATH") {
                        let directories = env::split_paths(&path);
                        for directory in directories {
                            let path = directory.join(args[1]);
                            if path.exists() {
                                #[cfg(unix)]
                                {
                                    let metadata = fs::metadata(&path).unwrap();
                                    let mode = metadata.permissions().mode();
                                    if mode & 0o111 != 0 {
                                        println!("{} is {}", args[1], &path.display());
                                    }
                                }
                                #[cfg(not(unix))]
                                {
                                    // On Windows, just check if file exists (simplified)
                                    println!("{} is {}", args[1], &path.display());
                                }
                            }
                        }
                    } else {
                        println!("{}: not found", args[1]);
                    }
                }
            }
            _ => {
                println!("{command}: command not found");
            }
        }
    }
}
