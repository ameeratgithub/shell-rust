#[cfg(unix)]
use std::fs::Metadata;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdout, Command as ProcessCommand, Stdio},
    str::FromStr,
    sync::atomic::Ordering,
};

use crate::{
    HISTORY_WRITTEN_COUNT, RL_EDITOR,
    keywords::KEYWORDS,
    lexer::RedirectionOperator,
    parser::{AstNode, Command, Redirection},
};

#[derive(Debug, PartialEq, Eq)]
pub enum HistoryFileAction {
    Read,
    Write,
    Append,
    None,
}

pub enum PipeState {
    ChildOutput(ChildStdout),
    BuiltinString(String),
}

pub enum VMError {
    Exit,
    Other(String),
}
pub struct VM {
    previous_state: Option<PipeState>,
    total_executed: usize,
    total_commands: usize,
    ast: AstNode,
}

impl VM {
    pub fn new(ast: AstNode) -> Self {
        Self {
            previous_state: None,
            total_executed: 0,
            total_commands: 0,
            ast,
        }
    }
    pub fn execute(&mut self) -> Result<(), VMError> {
        let ast = &mut self.ast.clone();
        match ast {
            // AstNode::SimpleCommand(command) => {
            //     VM::execute_command(command)?;
            // }

            // AstNode::BinaryOp { op: _, left, right } => {
            //     VM::execute(*left)?;
            //     VM::execute(*right)?;
            // }
            AstNode::Commands(commands) => {
                self.total_commands = commands.len();
                self.total_executed = 0;
                let mut children = vec![];

                for command in commands {
                    if let Some(child) = self.execute_command(command)? {
                        children.push(child);
                    }
                    self.total_executed += 1;
                }

                for mut child in children {
                    let _ = child.wait();
                }
            }
        }

        Ok(())
    }

    fn execute_command(&mut self, command: &mut Command) -> Result<Option<Child>, VMError> {
        let program = command.program.as_str();
        let args = &mut command.args;

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
                "exit" => {
                    let _ =
                        VM::get_history(None, env::var("HISTFILE").ok(), HistoryFileAction::Append);
                    return Err(VMError::Exit);
                }
                "history" => VM::execute_history(args),
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
                        } else if self.total_executed + 1 < self.total_commands {
                            self.previous_state = Some(PipeState::BuiltinString(output_string))
                        } else {
                            println!("{output_string}");
                        }
                    }
                }
                Err(e) => {
                    if !e.is_empty() {
                        if let Some(mut f) = file
                            && write_error_to_file
                        {
                            let _ = writeln!(f, "{}", e);
                        } else {
                            eprintln!("{e}");
                        }
                    }
                }
            }

            return Ok(None);
        }

        let external_result = self
            .execute_program(program, args, redirection, write_error_to_file)
            .map_err(VMError::Other)?;

        Ok(external_result)
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

    fn execute_echo(args: &Vec<String>) -> Result<String, String> {
        let output = args
            .iter()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(format!("{output}"))
    }

    fn execute_history(args: &Vec<String>) -> Result<String, String> {
        let mut iter = args.iter();
        let args = if let Some(arg) = iter.next().take() {
            if arg == "-r" {
                (None, iter.next().cloned(), HistoryFileAction::Read)
            } else if arg == "-w" {
                (None, iter.next().cloned(), HistoryFileAction::Write)
            } else if arg == "-a" {
                (None, iter.next().cloned(), HistoryFileAction::Append)
            } else if let Ok(skip) = usize::from_str(arg) {
                (Some(skip), None, HistoryFileAction::None)
            } else {
                (None, None, HistoryFileAction::None)
            }
        } else {
            (None, None, HistoryFileAction::None)
        };

        VM::get_history(args.0, args.1, args.2)
    }

    fn get_history(
        from_end: Option<usize>,
        file_path: Option<String>,
        action: HistoryFileAction,
    ) -> Result<String, String> {
        let raw_lines = if let Some(p) = file_path.clone() {
            if action == HistoryFileAction::Read {
                read_history_from_file(p)?
            } else {
                let lines = read_history_from_memory();

                if action == HistoryFileAction::Write {
                    write_history_to_file(p, lines.clone())?;
                    return Ok(String::new());
                } else if action == HistoryFileAction::Append {
                    let lines = read_history_from_memory();
                    let written_so_far = HISTORY_WRITTEN_COUNT.load(Ordering::Relaxed);

                    if written_so_far < lines.len() {
                        let new_lines = lines[written_so_far..].to_vec();

                        write_history_to_file(p, new_lines)?;

                        HISTORY_WRITTEN_COUNT.store(lines.len(), Ordering::Relaxed);
                    }

                    return Ok(String::new());
                }

                lines
            }
        } else {
            if let Ok(file_path) = env::var("HISTFILE") {
                let mut file_content = read_history_from_file(file_path)?;
                let mut memory_content = read_history_from_memory();
                file_content.append(&mut memory_content);
                file_content
            } else {
                read_history_from_memory()
            }
        };

        if let Some(file_name) = file_path {
            if action == HistoryFileAction::Read {
                let mut editor = RL_EDITOR.lock().unwrap();
                let _ = editor.clear_history();

                let _ = editor.add_history_entry(format!("history -r {}", file_name));

                for line in raw_lines {
                    let _ = editor.add_history_entry(line);
                }

                return Ok(String::new());
            }
        }

        let skip_count = match from_end {
            Some(n) => raw_lines.len().saturating_sub(n),
            None => 0,
        };

        let history_string = raw_lines
            .into_iter()
            .enumerate()
            .skip(skip_count)
            .map(|(index, item)| format!("    {}  {}", index + 1, item))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(history_string)
    }

    fn print_working_directory() -> Result<String, String> {
        match env::current_dir() {
            Ok(curr_dir) => Ok(format!("{}", curr_dir.display())),
            Err(e) => Err(e.to_string()),
        }
    }

    fn check_type_of_command(args: &Vec<String>) -> Result<String, String> {
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

    fn change_directory(args: &Vec<String>) -> Result<String, String> {
        if (args.first().is_none() || args[0] == "~")
            && let Ok(home_path) = env::var("HOME")
        {
            env::set_current_dir(home_path)
                .map_err(|_| String::from("Can't set current directory"))?;
            return Ok(String::new());
        }

        let Ok(path) = PathBuf::from_str(&args[0]);
        if path.is_dir() {
            env::set_current_dir(path).map_err(|_| String::from("Can't set current directory"))?;
            Ok(String::new())
        } else {
            Ok(format!("cd: {}: No such file or directory", path.display()))
        }
    }

    fn execute_program(
        &mut self,
        program: &str,
        args: &mut Vec<String>,
        redirection: Option<&Redirection>,
        is_error: bool,
    ) -> Result<Option<std::process::Child>, String> {
        if !program.contains(" ") {
            check_executable_file_exists_in_paths(program)
                .ok_or_else(|| format!("{program}: command not found"))?;
        }

        let (stdout_cfg, stderr_cfg) = if let Some(rd) = redirection {
            let file_stdio = VM::get_file_for_redirection(rd)
                .map(Stdio::from)
                .unwrap_or(self.get_default_stdio());

            if is_error {
                (self.get_default_stdio(), file_stdio)
            } else {
                (file_stdio, self.get_default_stdio())
            }
        } else {
            (self.get_default_stdio(), self.get_default_stdio())
        };

        let mut builtin_text_to_write = None;

        let default_stdin = match self.previous_state.take() {
            Some(PipeState::BuiltinString(text)) => {
                builtin_text_to_write = Some(text);
                Stdio::piped()
            }
            Some(PipeState::ChildOutput(child_stdout)) => Stdio::from(child_stdout),
            None => Stdio::inherit(),
        };

        let mut command = ProcessCommand::new(program)
            .args(&mut args[0..])
            .stdin(default_stdin)
            .stdout(stdout_cfg)
            .stderr(stderr_cfg)
            .spawn()
            .map_err(|_| format!("{program}: command not found"))?;

        if let Some(text) = builtin_text_to_write {
            if let Some(mut child_stdin) = command.stdin.take() {
                let _ = child_stdin.write_all(text.as_bytes());
                let _ = child_stdin.write_all(b"\n");
            }
        }

        if let Some(stdout) = command.stdout.take() {
            self.previous_state = Some(PipeState::ChildOutput(stdout));
        }

        Ok(Some(command))
    }

    fn get_default_stdio(&self) -> Stdio {
        if self.total_executed + 1 == self.total_commands {
            Stdio::inherit()
        } else {
            Stdio::piped()
        }
    }
}

fn read_history_from_memory() -> Vec<String> {
    let editor = RL_EDITOR.lock().unwrap();
    editor
        .history()
        .iter()
        .map(|entry| entry.to_string())
        .collect()
}

fn write_history_to_file(path: String, data: Vec<String>) -> Result<(), String> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    let mut writer = BufWriter::new(file);

    for line in data {
        writeln!(writer, "{}", line).map_err(|e| e.to_string())?;
    }

    writer.flush().map_err(|e| e.to_string())?;

    Ok(())
}

fn read_history_from_file(path: String) -> Result<Vec<String>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let lines_iter = reader.lines();

    let result = lines_iter
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(result)
}

fn check_executable_file_exists_in_paths(file: &str) -> Option<String> {
    if let Ok(paths) = env::var("PATH") {
        let directories = env::split_paths(&paths);
        for directory in directories {
            let path = directory.join(file);
            if path.exists() {
                let metadata = fs::metadata(&path);

                // We may not have permissions to access the directory, so we want to safely
                // ignore the error
                if let Ok(m) = metadata
                    && is_executable(&m)
                {
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
