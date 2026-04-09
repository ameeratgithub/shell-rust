use std::{
    fmt::{Display, Formatter},
    iter::Peekable,
    str::Chars,
};

use crate::keywords::REDIRECTION_OPERATORS;

#[derive(Debug, PartialEq, Eq)]
pub enum LexerError {
    UnterminatedString,
    Redirection(String),
    Other(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum WordPart {
    Unquoted(String),
    SingleQuoted(String),
    DoubleQuoted(String),
}

impl Display for WordPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WordPart::Unquoted(val) | WordPart::SingleQuoted(val) | WordPart::DoubleQuoted(val) => {
                write!(f, "{val}")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RedirectionOperator {
    Overwrite,
    OverwriteError,
    Append,
    AppendError,
    Input,
}

impl Display for RedirectionOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overwrite => write!(f, ">"),
            Self::OverwriteError => write!(f, "2>"),
            Self::Append => write!(f, ">>"),
            Self::AppendError => write!(f, "2>>"),
            Self::Input => write!(f, "<"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ControlOperator {
    Pipe,     // |
    And,      // &&
    Or,       // ||
    Sequence, // ;
}

impl Display for ControlOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pipe => write!(f, "|"),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::Sequence => write!(f, ";"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Word(Vec<WordPart>),
    Redirection(RedirectionOperator),
    Operator(ControlOperator),
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
                    word_str.push_str(&part.to_string());
                }

                write!(f, "{word_str}")
            }

            TokenType::Redirection(op) => write!(f, "{op}"),
            TokenType::Operator(op) => write!(f, "{op}"),
        }
    }
}

#[allow(dead_code)]
pub struct TokenList(pub Vec<Token>);

impl Display for TokenList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted_tokens: Vec<String> = self.0.iter().map(|token| token.to_string()).collect();

        write!(f, "{}", formatted_tokens.join(","))
    }
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(command: &'a str) -> Self {
        Self {
            chars: command.chars().peekable(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some(&char) = self.chars.peek() {
            if char.is_whitespace() {
                self.chars.next();
                continue;
            }

            if char == '|' {
                self.chars.next();
                let token = Token::new(TokenType::Operator(ControlOperator::Pipe));
                tokens.push(token);
                continue;
            }

            let (is_redirection_operator, is_error) = self.is_a_redirection_operator(&char);
            if is_redirection_operator {
                let c = self
                    .chars
                    .peek()
                    .ok_or(LexerError::Redirection(String::from(
                        "Invalid Redirection operator",
                    )))
                    .cloned()?;

                let redirection_operator = self.get_redirection_operator(&c, is_error);
                let token = Token::new(TokenType::Redirection(redirection_operator));
                tokens.push(token);

                self.chars.next();

                continue;
            }

            tokens.push(self.scan_word()?);
        }

        Ok(tokens)
    }

    fn scan_word(&mut self) -> Result<Token, LexerError> {
        let mut word_parts = vec![];

        while let Some(char) = self.chars.peek() {
            if char.is_whitespace() || REDIRECTION_OPERATORS.get(char).is_some() {
                return Ok(Token::new(TokenType::Word(word_parts)));
            }

            match *char {
                '\'' => word_parts.push(self.scan_single_quoted_word()?),
                '"' => word_parts.push(self.scan_double_quoted_word()?),
                _ => word_parts.push(self.scan_unquoted_word()?),
            }
        }

        Ok(Token::new(TokenType::Word(word_parts)))
    }

    fn scan_single_quoted_word(&mut self) -> Result<WordPart, LexerError> {
        self.chars.next();

        let mut word_part = String::new();

        while let Some(char) = self.chars.next() {
            if char == '\'' {
                return Ok(WordPart::SingleQuoted(word_part));
            }

            word_part.push(char);
        }

        Err(LexerError::UnterminatedString)
    }

    fn scan_double_quoted_word(&mut self) -> Result<WordPart, LexerError> {
        self.chars.next();

        let mut word_part = String::new();

        while let Some(char) = self.chars.next() {
            if char == '\\' {
                let next_char = self
                    .chars
                    .next()
                    .ok_or_else(|| LexerError::UnterminatedString)?;
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

    fn scan_unquoted_word(&mut self) -> Result<WordPart, LexerError> {
        let mut word_part = String::new();

        while let Some(char) = self.chars.peek() {
            if *char == '\\' {
                self.chars.next();
            } else if *char == '\''
                || *char == '"'
                || char.is_whitespace()
                || REDIRECTION_OPERATORS.contains(char)
            {
                return Ok(WordPart::Unquoted(word_part));
            }

            word_part.push(
                self.chars
                    .next()
                    .ok_or(LexerError::Other(String::from("Character expected")))?,
            );
        }

        Ok(WordPart::Unquoted(word_part))
    }

    /// @returns (is_operator, is_error)
    fn is_a_redirection_operator(&mut self, char: &char) -> (bool, bool) {
        if *char == '1' || *char == '2' {
            let mut look_ahead: Peekable<Chars<'a>> = self.chars.clone();
            look_ahead.next();

            if let Some(c) = look_ahead.peek()
                && REDIRECTION_OPERATORS.contains(c)
            {
                self.chars.next();
                return (true, *char == '2');
            }
        }

        (REDIRECTION_OPERATORS.contains(char), false)
    }

    fn get_redirection_operator(&mut self, char: &char, is_error: bool) -> RedirectionOperator {
        if *char == '>' {
            let mut look_ahead: Peekable<Chars<'a>> = self.chars.clone();
            look_ahead.next();

            if let Some(c) = look_ahead.peek()
                && *c == '>'
            {
                self.chars.next();
                if is_error {
                    RedirectionOperator::AppendError
                } else {
                    RedirectionOperator::Append
                }
            } else {
                if is_error {
                    RedirectionOperator::OverwriteError
                } else {
                    RedirectionOperator::Overwrite
                }
            }
        } else {
            RedirectionOperator::Input
        }
    }
}
