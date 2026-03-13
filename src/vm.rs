#[cfg(unix)]
use std::fs::Metadata;
use std::{env, fs, path::PathBuf, process::Command as ProcessCommand, str::FromStr};

use crate::{
    keywords::KEYWORDS,
    parser::{AstNode, Command},
};

pub enum VMError {
    Exit,
    Other(String),
}
pub struct VM;
impl VM {
    pub fn execute(ast: AstNode) -> Result<(), VMError> {
        match ast {
            AstNode::SimpleCommand(command) => {
                VM::execute_command(command)?;
            }

            AstNode::Pipeline { left, right } => {
                VM::execute(*left)?;
                VM::execute(*right)?;
            }
        }

        Ok(())
    }

    fn execute_command(command: Command) -> Result<(), VMError> {
        let program = command.program.as_str();
        let mut args = command.args;
        match program {
            "echo" => VM::execute_echo(args),
            "exit" => return Err(VMError::Exit),
            "pwd" => VM::print_working_directory(),
            "cd" => VM::change_directory(args),
            "type" => VM::check_type_of_command(args),
            _ => VM::execute_program(program, &mut args),
        }

        Ok(())
    }

    fn execute_echo(args: Vec<String>) {
        let output = args
            .iter()
            .skip(1)
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{output}");
    }

    fn print_working_directory() {
        if let Ok(curr_dir) = env::current_dir() {
            println!("{}", curr_dir.display());
        }
    }

    fn check_type_of_command(args: Vec<String>) {
        let first_arg = args[0].as_str();
        if KEYWORDS.contains(first_arg) {
            println!("{} is a shell builtin", first_arg)
        } else {
            if let Some(path) = check_executable_file_exists_in_paths(first_arg) {
                println!("{} is {}", first_arg, path);
            } else {
                eprintln!("{}: not found", first_arg);
            }
        }
    }
    fn change_directory(args: Vec<String>) {
        if (args.first().is_none() || args[0] == "~")
            && let Ok(home_path) = env::var("HOME")
        {
            env::set_current_dir(home_path).unwrap();
            return;
        }

        let Ok(path) = PathBuf::from_str(&args[0]);
        if path.is_dir() {
            env::set_current_dir(path).unwrap();
        } else {
            println!("cd: {}: No such file or directory", path.display())
        }
    }

    fn execute_program(program: &str, args: &mut Vec<String>) {
        if !program.contains(" ") {
            let executable_path = check_executable_file_exists_in_paths(program);
            if executable_path.is_none() {
                eprintln!("{program}: command not found");
                return;
            }
        }

        let execution_result = ProcessCommand::new(program).args(&mut args[0..]).status();
        if let Err(_) = execution_result {
            eprintln!("{program}: command not found");
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

                if is_executable(&metadata) {
                    return path.to_str().map(|str| str.to_owned());
                }
            }
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(metadata: &Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(windows)]
fn is_executable(_metadata: &Metadata) -> bool {
    // windows doesn't use permission bits for executables.
    true
}
