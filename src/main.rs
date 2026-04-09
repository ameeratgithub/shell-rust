use std::{env, fs};

use rustyline::{
    CompletionType, Config, Editor, Helper, Highlighter, Hinter, Validator,
    completion::{Completer, Pair, extract_word},
    error::ReadlineError,
};

use crate::{
    lexer::{Lexer, LexerError},
    parser::Parser,
    vm::{VM, VMError},
};

mod keywords;
mod lexer;
mod parser;
mod vm;

#[derive(Helper, Highlighter, Hinter, Validator)]
struct ShellHelper {
    commands: Vec<String>,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let (start, word) = extract_word(line, pos, None, |c| c == ' ');

        let matches: Vec<Pair> = self
            .commands
            .iter()
            .filter(|cmd| cmd.starts_with(word))
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: format!("{} ", cmd),
            })
            .collect();

        Ok((start, matches))
    }
}

fn get_path_files() -> Vec<String> {
    let Some(path_var) = env::var_os("PATH") else {
        return Vec::new();
    };

    env::split_paths(&path_var)
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_file()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}
fn main() {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config).unwrap();

    let mut built_in_commands = vec!["echo".to_string(), "exit".to_string()];
    built_in_commands.extend(get_path_files());
    built_in_commands.sort();
    built_in_commands.dedup();

    let helper = ShellHelper {
        commands: built_in_commands,
    };
    rl.set_helper(Some(helper));

    loop {
        let mut command: String = String::new();

        match rl.readline("$ ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                command.push_str(&line);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }

        command = command.trim().to_string();
        if command.is_empty() {
            continue;
        }

        let tokens = loop {
            let mut lexer = Lexer::new(&command);
            match lexer.scan_tokens() {
                Ok(tokens) => break tokens,
                Err(LexerError::UnterminatedString) => match rl.readline("> ") {
                    Ok(line) => {
                        rl.add_history_entry(line.as_str()).unwrap();
                        command.push('\n');
                        command.push_str(&line);
                    }
                    Err(_) => return,
                },
                Err(_) => {
                    eprintln!("Lexer Error");
                    return;
                }
            }
        };

        let parse_result = Parser::parse(tokens);
        match parse_result {
            Ok(ast) => {
                let mut vm = VM::new(ast);
                match vm.execute() {
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
}
