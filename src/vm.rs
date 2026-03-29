#[cfg(unix)]
use std::fs::Metadata;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command as ProcessCommand, Stdio},
    str::FromStr,
};

use crate::{
    keywords::KEYWORDS,
    lexer::RedirectionOperator,
    parser::{AstNode, Command, Redirection},
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

            AstNode::BinaryOp { op: _, left, right } => {
                VM::execute(*left)?;
                VM::execute(*right)?;
            }
        }

        Ok(())
    }

    fn execute_command(command: Command) -> Result<(), VMError> {
        let program = command.program.as_str();
        let mut args = command.args;

        let redirection = command.redirections.first();

        let mut write_error_to_file = false;
        let mut file = None;

        if let Some(rd) = redirection {
            write_error_to_file = VM::is_error_redirection(rd);
            file = VM::get_file_for_redirection(rd);
        }

        if KEYWORDS.contains(program) {
            let output_result = match program {
                "echo" => VM::execute_echo(args),
                "exit" => return Err(VMError::Exit),
                "pwd" => VM::print_working_directory(),
                "cd" => VM::change_directory(args),
                "type" => VM::check_type_of_command(args),
                _ => unreachable!(),
            };

            match output_result {
                Ok(output_string) => {
                    if !output_string.is_empty() {
                        if let Some(mut f) = file
                            && !write_error_to_file
                        {
                            let _ = writeln!(f, "{}", output_string);
                        } else {
                            println!("{output_string}");
                        }
                    }
                }
                Err(e) => {
                    if !e.is_empty() {
                        if let Some(mut f) = file && write_error_to_file {
                            let _ = writeln!(f, "{}", e);
                        } else {
                            eprintln!("{e}");
                        }
                    }
                }
            }

            return Ok(());
        }

        let external_result =
            VM::execute_program(program, &mut args, redirection, write_error_to_file);
        if let Err(e) = external_result {
            eprintln!("{}", e);
        }

        Ok(())
    }

    fn is_error_redirection(redirection: &Redirection) -> bool {
        redirection.op == RedirectionOperator::AppendError
            || redirection.op == RedirectionOperator::OverwriteError
    }

    fn get_file_for_redirection(redirection: &Redirection) -> Option<File> {
        let path = Path::new(&redirection.file);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if redirection.op == RedirectionOperator::Overwrite {
            File::create(&redirection.file).ok()
        } else {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&redirection.file)
                .ok()
        }
    }

    fn execute_echo(args: Vec<String>) -> Result<String, String> {
        let output = args
            .iter()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(format!("{output}"))
    }

    fn print_working_directory() -> Result<String, String> {
        match env::current_dir() {
            Ok(curr_dir) => Ok(format!("{}", curr_dir.display())),
            Err(e) => Err(e.to_string()),
        }
    }

    fn check_type_of_command(args: Vec<String>) -> Result<String, String> {
        let first_arg = args[0].as_str();
        if KEYWORDS.contains(first_arg) {
            Ok(format!("{} is a shell builtin", first_arg))
        } else {
            if let Some(path) = check_executable_file_exists_in_paths(first_arg) {
                Ok(format!("{} is {}", first_arg, path))
            } else {
                Err(format!("{}: not found", first_arg))
            }
        }
    }

    fn change_directory(args: Vec<String>) -> Result<String, String> {
        if (args.first().is_none() || args[0] == "~")
            && let Ok(home_path) = env::var("HOME")
        {
            env::set_current_dir(home_path).unwrap();
            return Ok(String::new());
        }

        let Ok(path) = PathBuf::from_str(&args[0]);
        if path.is_dir() {
            env::set_current_dir(path).unwrap();
            Ok(String::new())
        } else {
            Ok(format!("cd: {}: No such file or directory", path.display()))
        }
    }

    fn execute_program(
        program: &str,
        args: &mut Vec<String>,
        redirection: Option<&Redirection>,
        is_error: bool,
    ) -> Result<String, String> {
        if !program.contains(" ") {
            check_executable_file_exists_in_paths(program)
                .ok_or_else(|| format!("{program}: command not found"))?;
        }

        let (stdout_cfg, stderr_cfg) = if let Some(rd) = redirection {
            let file_stdio = VM::get_file_for_redirection(rd)
                .map(Stdio::from)
                .unwrap_or_else(Stdio::inherit);

            if is_error {
                (Stdio::inherit(), file_stdio)
            } else {
                (file_stdio, Stdio::inherit())
            }
        } else {
            (Stdio::inherit(), Stdio::inherit())
        };

        ProcessCommand::new(program)
            .args(&mut args[0..])
            .stdout(stdout_cfg)
            .stderr(stderr_cfg)
            .status()
            .map_err(|_| format!("{program}: command not found"))?;

        Ok(String::new())
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
