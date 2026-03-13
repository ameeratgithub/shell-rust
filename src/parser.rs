use crate::lexer::{Token, TokenType};

#[derive(Debug)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn new(program: String, args: Vec<String>) -> Self {
        Self { program, args }
    }
}

#[derive(Debug)]
pub enum AstNode {
    SimpleCommand(Command),
    Pipeline {
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
}

impl AstNode {
    pub fn new(command: Command) -> Self {
        AstNode::SimpleCommand(command)
    }
}

#[derive(Debug)]
pub enum ParserError {
    Other(String),
}

pub struct Parser;

impl Parser {
    pub fn parse(tokens: Vec<Token>) -> Result<AstNode, ParserError> {
        if tokens.is_empty() {
            return Err(ParserError::Other("Tokens are empty".to_string()));
        }

        let program = tokens.first().unwrap().to_string();

        let args = tokens
            .iter()
            .skip(1)
            .map_while(|t| match t.ty {
                TokenType::Word(_) => Some(t.to_string()),
            })
            .collect::<Vec<String>>();

        let command = Command::new(program, args);
        let ast = AstNode::new(command);
        Ok(ast)
    }
}
