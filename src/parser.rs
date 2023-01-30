use std::{collections::HashMap, string::ParseError};

use crate::lexer::error::{ErrorType, ParseResult};
pub use crate::lexer::{Lexeme, Lexer, Operator};

const MAX_PRECEDENCE: u8 = 3;

#[derive(Debug)]
pub enum Expression {
    Integer(u64),
    BinaryOperation(Operator, Box<Expression>, Box<Expression>),
    UnaryOperation(Operator, Box<Expression>),
    Sequence(Vec<Expression>),
}

pub struct Parser {
    lexer: Lexer,
    token: Lexeme,
    precedence_lookup: HashMap<Operator, u8>,
}

impl Parser {
    pub fn new(program: Vec<u8>) -> Self {
        let mut precedence_lookup = HashMap::new();

        precedence_lookup.insert(Operator::Semicolon, 3);
        precedence_lookup.insert(Operator::Plus, 2);
        precedence_lookup.insert(Operator::Minus, 2);
        precedence_lookup.insert(Operator::Asterisk, 1);

        Self {
            lexer: Lexer::new(program),
            token: Lexeme::Eof,
            precedence_lookup,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Expression> {
        self.advance()?;

        self.binary_operation(MAX_PRECEDENCE)
    }

    fn binary_operation(&mut self, precedence: u8) -> ParseResult<Expression> {
        if precedence == 0 {
            return self.term();
        }

        let mut expr = self.binary_operation(precedence - 1)?;

        // create a list of statements if we are at semicolon level
        if precedence
            == *self
                .precedence_lookup
                .get(&Operator::Semicolon)
                .expect("Semicolon missing from precedences")
        {
            expr = Expression::Sequence(vec![expr]);
        }

        while let Lexeme::Operator(op) = self.token {
            if let Some(&op_precedence) = self.precedence_lookup.get(&op) {
                if op_precedence == precedence {
                    self.advance()?;
                    if let Expression::Sequence(expr_seq) = &mut expr {
                        expr_seq.push(self.binary_operation(precedence - 1)?); //todo handle EOF
                                                                               //here
                    } else {
                        expr = Expression::BinaryOperation(
                            op,
                            Box::new(expr),
                            Box::new(self.binary_operation(precedence - 1)?),
                        );
                    }
                } else {
                    break;
                }
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expression> {
        match self.token {
            Lexeme::Integer(i) => {
                self.advance()?;
                Ok(Expression::Integer(i))
            }

            Lexeme::Operator(Operator::Minus) => {
                self.advance()?;
                Ok(Expression::UnaryOperation(
                    Operator::Minus,
                    Box::new(self.term()?),
                ))
            }

            Lexeme::Operator(Operator::OpenParenthesis) => {
                self.advance()?;
                let expr = self.binary_operation(MAX_PRECEDENCE)?;

                if let Lexeme::Operator(Operator::CloseParenthesis) = self.token {
                    self.advance()?;
                    Ok(expr)
                } else {
                    Err(ErrorType::ParseError("Unclosed parenthesis".to_string()))
                }
            }

            _ => Err(ErrorType::ParseError(format!(
                "Expected term, recieved {:?}",
                self.token
            ))),
        }
    }

    fn advance(&mut self) -> ParseResult<()> {
        self.token = self.lexer.next()?;
        Ok(())
    }
}
