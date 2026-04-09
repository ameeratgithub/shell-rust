use crate::lexer::{ControlOperator, RedirectionOperator, Token, TokenType};

#[derive(Debug, Clone)]
pub struct Redirection {
    pub op: RedirectionOperator,
    pub file: String,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub redirections: Vec<Redirection>,
}

impl Command {
    pub fn new(program: String, args: Vec<String>, redirections: Vec<Redirection>) -> Self {
        Self {
            program,
            args,
            redirections,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AstNode {
    // BinaryOp {
    //     op: ControlOperator,
    //     left: Box<AstNode>,
    //     right: Box<AstNode>,
    // },
    Commands(Vec<Command>),
}

#[derive(Debug)]
pub enum ParserError {
    Other(String),
}

pub struct Parser {}

impl Parser {
    pub fn parse(tokens: Vec<Token>) -> Result<AstNode, ParserError> {
        if tokens.is_empty() {
            return Err(ParserError::Other("Tokens are empty".to_string()));
        }

        let command_slice =
            tokens.split(|t| matches!(t.ty, TokenType::Operator(ControlOperator::Pipe)));

        let mut commands = vec![];

        for command_tokens in command_slice {
            let command = Parser::build_command(command_tokens)?;
            commands.push(command);
        }

        Ok(AstNode::Commands(commands))
    }
    fn build_command(tokens: &[Token]) -> Result<Command, ParserError> {
        let program = tokens
            .first()
            .ok_or(ParserError::Other(String::from("program name is expected")))?
            .to_string();

        let mut args: Vec<String> = vec![];
        let mut redirections = vec![];

        let mut iter = tokens.iter().skip(1);
        while let Some(token) = iter.next() {
            match &token.ty {
                TokenType::Word(_) => args.push(token.to_string()),
                TokenType::Redirection(op) => {
                    if let Some(file_token) = iter.next() {
                        redirections.push(Redirection {
                            op: op.clone(),
                            file: file_token.to_string(),
                        });
                    } else {
                        eprintln!("syntax error: expected file name after redirection operator");
                    }
                }
                _ => {}
            }
        }

        Ok(Command::new(program, args, redirections))
    }
}
