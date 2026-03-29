use std::{env, fs};

use rustyline::{
    Editor, Helper, Highlighter, Hinter, Validator,
    completion::{Completer, Pair, extract_word},
    error::ReadlineError,
};

use crate::{
    lexer::{Lexer, LexerError},
    parser::Parser,
    vm::{VM, VMError},
};

#[allow(unused_imports)]
mod eval;
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
        let mut matches = Vec::new();

        for cmd in &self.commands {
            if cmd.starts_with(word) {
                matches.push(Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                });
            }
        }

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
        .map(|f|f+" ")
        .collect()
}
fn main() {
    let mut rl = Editor::new().unwrap();
  
    let mut built_in_commands = vec!["echo ".to_string(), "exit ".to_string()];
    built_in_commands.extend(get_path_files());

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
                let vm_result = VM::execute(ast);
                match vm_result {
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
    // eval::run();
}
