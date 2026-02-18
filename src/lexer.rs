use std::{iter::Peekable, str::CharIndices};

#[derive(Debug)]
pub enum LexerError {
    UnterminatedString,
    Other(String),
}
pub enum TokenType {
    Word,
    Literal,
    Mixed,
    Compound,
    CompoundLiteral,
    // Metacharacters
    // Pipe,
    // RedirectOut, // >
    // RedirectIn, // <
    // Semicolon, // ;
    Eof,
}
pub struct Token {
    ty: TokenType,
    start: usize,
    len: usize,
}

impl Token {
    pub fn new(ty: TokenType, start: usize, len: usize) -> Self {
        Self { ty, start, len }
    }
}

pub struct Lexer {
    curr_offset: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Self { curr_offset: 0 }
    }

    pub fn scan(&self, command: &str) -> Result<Vec<Token>, LexerError> {
        let chars = &mut command.char_indices().peekable();
        let tokens = self.scan_tokens(chars);
        Ok(tokens)
    }

    fn scan_tokens(&self, chars: &mut Peekable<CharIndices>) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some((offset, char)) = chars.peek() {
            if char.is_whitespace() {
                continue;
            }

            if *char == '\'' || *char == '"' {
            } else {
                tokens.push(self.scan_word(chars));
            }
        }
        tokens
    }

    fn scan_word(&self, chars: &mut Peekable<CharIndices>) -> Token {
        let mut len = 0;
    }
    fn scan_literal(&self, chars: &mut Peekable<CharIndices>) -> Token {
        let mut len = 0;
    }
}
