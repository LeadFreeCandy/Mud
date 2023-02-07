use std::collections::HashMap;

use crate::lexer::error::{ErrorType, MudResult};
pub use crate::lexer::{Lexeme, Lexer, Operator};
use once_cell::sync::Lazy; // TODO: figure out why it cannot be unsync

#[derive(Debug)]
pub enum Expression {
    Null,
    Integer(u64),
    Identifier(String),
    BinaryOperation(Operator, Box<Expression>, Box<Expression>), // TODO: probably get rid of expression composition as a binary operation
    UnaryOperation(Operator, Box<Expression>),
}

pub struct Parser {
    lexer: Lexer,
    token: Lexeme,
}


static PRECEDENCE_LOOKUP: Lazy<HashMap<Operator, u8>> = Lazy::new(|| {

    let mut precedence_lookup = HashMap::new();

    precedence_lookup.insert(Operator::Semicolon, 5);

    precedence_lookup.insert(Operator::Equals, 4);
    precedence_lookup.insert(Operator::Colon, 4);

    precedence_lookup.insert(Operator::LessThan, 3);
    precedence_lookup.insert(Operator::GreaterThan, 3);

    precedence_lookup.insert(Operator::Plus, 2);
    precedence_lookup.insert(Operator::Minus, 2);
    precedence_lookup.insert(Operator::Asterisk, 1);
    precedence_lookup
});

static MAX_PRECEDENCE: Lazy<u8> = Lazy::new(|| {
    *PRECEDENCE_LOOKUP.values().max().unwrap()
});

impl Parser {
    pub fn new(program: Vec<u8>) -> Self {
        Self {
            lexer: Lexer::new(program),
            token: Lexeme::Eof,
        }
    }

    pub fn parse(&mut self) -> MudResult<Expression> {
        self.advance()?;

        self.binary_operation(*MAX_PRECEDENCE)
    }

    fn binary_operation(&mut self, precedence: u8) -> MudResult<Expression> {
        if precedence == 0 {
            return self.term();
        }

        let mut expr = self.binary_operation(precedence - 1)?;

        while let Lexeme::Operator(op) = self.token {
            if let Some(&op_precedence) = PRECEDENCE_LOOKUP.get(&op) {
                if op_precedence == precedence {
                    self.advance()?;
                    expr = Expression::BinaryOperation(
                        op,
                        Box::new(expr),
                        Box::new(self.binary_operation(precedence - 1)?),
                    );
                } else {
                    break;
                }
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> MudResult<Expression> {
        match self.advance()? {
            Lexeme::Integer(i) => {
                Ok(Expression::Integer(i))
            }

            Lexeme::Identifier(s) => {
                Ok(Expression::Identifier(s))
            }

            //negate
            Lexeme::Operator(Operator::Minus) => {
                Ok(Expression::UnaryOperation(
                    Operator::Minus,
                    Box::new(self.term()?),
                ))
            }

            //print
            Lexeme::Operator(Operator::LessThan) => {
                Ok(Expression::UnaryOperation(
                    Operator::LessThan,
                    Box::new(self.term()?),
                ))
            }

            Lexeme::Operator(Operator::OpenParenthesis) => {
                let expr = self.binary_operation(*MAX_PRECEDENCE)?;

                if let Lexeme::Operator(Operator::CloseParenthesis) = self.token {
                    self.advance()?;
                    Ok(expr)
                } else {
                    Err(ErrorType::ParseError("Unclosed parenthesis".to_string()))
                }
            }

            Lexeme::Eof => Ok(Expression::Null),

            _ => Err(ErrorType::ParseError(format!(
                "Expected term, recieved {:?}",
                self.token
            ))),
        }
    }

    fn advance(&mut self) -> MudResult<Lexeme> {
        Ok(std::mem::replace(&mut self.token, self.lexer.next()?))
    }
}
