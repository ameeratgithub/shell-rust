use std::{
    fmt::{Display, Formatter},
    io::{self, Write},
    iter::Peekable,
    str::Chars,
};

#[derive(Debug, PartialEq, Eq)]
pub enum LexerError {
    UnterminatedString,
    Other(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum WordPart {
    Unquoted(String),
    SingleQuoted(String),
    DoubleQuoted(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Word(Vec<WordPart>),
}

#[derive(Debug)]
pub struct Token {
    pub ty: TokenType,
}

impl Token {
    pub fn new(ty: TokenType) -> Self {
        Self { ty }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.ty {
            TokenType::Word(word) => {
                let mut word_str = String::new();
                for part in word {
                    match part {
                        WordPart::Unquoted(val)
                        | WordPart::SingleQuoted(val)
                        | WordPart::DoubleQuoted(val) => {
                            word_str.push_str(val);
                        }
                    }
                }

                write!(f, "{word_str}")
            }
        }
    }
}

pub struct TokenList(pub Vec<Token>);

impl Display for TokenList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted_tokens: Vec<String> = self.0.iter().map(|token| token.to_string()).collect();

        write!(f, "{}", formatted_tokens.join(","))
    }
}

pub struct Lexer {
    _curr_offset: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Self { _curr_offset: 0 }
    }

    pub fn scan_until_complete(&self, command: &str) -> Vec<Token> {
        let tokens = self.scan(command);
        loop {
            if tokens.is_ok() {
                return tokens.unwrap();
            }

            if tokens.err().unwrap() == LexerError::UnterminatedString {
                print!("> ");
                io::stdout().flush().unwrap();

                let mut next_part = String::new();
                io::stdin().read_line(&mut next_part).unwrap();
                next_part.push_str(command);

                return self.scan_until_complete(&next_part);
            }

            return vec![];
        }
    }

    fn scan(&self, command: &str) -> Result<Vec<Token>, LexerError> {
        let chars = &mut command.chars().peekable();
        let tokens = self.scan_tokens(chars)?;
        Ok(tokens)
    }

    fn scan_tokens(&self, chars: &mut Peekable<Chars>) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some(char) = chars.peek() {
            if char.is_whitespace() {
                chars.next().unwrap();
                continue;
            }

            tokens.push(self.scan_word(chars)?);
        }

        Ok(tokens)
    }

    fn scan_word(&self, chars: &mut Peekable<Chars>) -> Result<Token, LexerError> {
        let mut word_parts = vec![];
        while let Some(char) = chars.peek() {
            if char.is_whitespace() {
                return Ok(Token::new(TokenType::Word(word_parts)));
            }

            if *char == '\'' {
                word_parts.push(self.scan_single_quoted_word(chars)?);
            } else if *char == '"' {
                word_parts.push(self.scan_double_quoted_word(chars)?);
            } else {
                word_parts.push(self.scan_unquoted_word(chars)?);
            }
        }

        Ok(Token::new(TokenType::Word(word_parts)))
    }

    fn scan_single_quoted_word(&self, chars: &mut Peekable<Chars>) -> Result<WordPart, LexerError> {
        chars.next();

        let mut word_part = String::new();

        while let Some(char) = chars.next() {
            if char == '\'' {
                return Ok(WordPart::SingleQuoted(word_part));
            }

            word_part.push(char);
        }

        Err(LexerError::UnterminatedString)
    }

    fn scan_double_quoted_word(&self, chars: &mut Peekable<Chars>) -> Result<WordPart, LexerError> {
        chars.next();

        let mut word_part = String::new();

        while let Some(char) = chars.next() {
            if char == '\\' {
                let next_char = chars.next().ok_or_else(|| LexerError::UnterminatedString)?;
                if next_char != '"' && next_char != '\\' {
                    word_part.push(char);
                }
                word_part.push(next_char);
            } else if char == '"' {
                return Ok(WordPart::DoubleQuoted(word_part));
            } else {
                word_part.push(char);
            }
        }

        Err(LexerError::UnterminatedString)
    }

    fn scan_unquoted_word(&self, chars: &mut Peekable<Chars>) -> Result<WordPart, LexerError> {
        let mut word_part = String::new();

        while let Some(char) = chars.peek() {
            if *char == '\\' {
                chars.next();
            } else if *char == '\'' || *char == '"' || char.is_whitespace() {
                return Ok(WordPart::Unquoted(word_part));
            }

            word_part.push(chars.next().unwrap());
        }

        Ok(WordPart::Unquoted(word_part))
    }
}
