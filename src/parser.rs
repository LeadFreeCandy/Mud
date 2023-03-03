use std::{collections::HashMap, string::ParseError};

use crate::lexer::{error::{ErrorType, MudResult}, Keyword};
pub use crate::lexer::{Lexeme, Lexer, Operator};
use once_cell::sync::Lazy; // TODO: figure out why it cannot be unsync

#[derive(Debug)]
pub enum Expression {
    Null,
    Integer(u64),
    Identifier(String),
    BinaryOperation { op: Operator, lhs: Box<Expression>, rhs: Box<Expression> }, // TODO: probably get rid of expression composition as a binary operation
    UnaryOperation { op: Operator, oprand: Box<Expression> },
    Block(Box<Expression>),
    IfElse { condition: Box<Expression>, on_if: Box<Expression>, on_else: Box<Expression> },
    While { condition: Box<Expression>, body: Box<Expression> },
    Function { args: Vec<Expression>, return_type: Box<Expression>, body: Box<Expression> }
}

pub struct Parser {
    lexer: Lexer,
    lexeme: Lexeme,
}


static PRECEDENCE_LOOKUP: Lazy<HashMap<Operator, u8>> = Lazy::new(|| {
    let mut precedence_lookup = HashMap::new();

    precedence_lookup.insert(Operator::Semicolon, 5);

    precedence_lookup.insert(Operator::ColonEquals, 4);
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

macro_rules! expect_lexeme {
    ($self:ident, $lexeme:pat) => {
        match &$self.lexeme {
            $lexeme => { $self.advance()?; },
            t => return Err(ErrorType::ParseError(format!("Expected lexeme {} but got {:?}", stringify!($lexeme), t))),
        }
    };
}

impl Parser {
    pub fn new(program: Vec<u8>) -> Self {
        Self {
            lexer: Lexer::new(program),
            lexeme: Lexeme::Eof,
        }
    }

    pub fn parse(&mut self) -> MudResult<Expression> {
        self.advance()?;
        let expr = self.expression();
        if let Lexeme::Eof = self.lexeme {
            expr
        }
        else {
            Err(ErrorType::ParseError("Expected EOF but got some lexeme".to_string()))
        }
    }

    fn expression(&mut self) -> MudResult<Expression> {
        self.binary_operation(*MAX_PRECEDENCE)
    }

    fn is_block(expr: &Expression) -> bool {
        if let Expression::Block(_) = expr {
            return true;
        }

        return false;
    }

    fn ifelse(&mut self) -> MudResult<Expression> {
        // assume `if` has already been consumed
        fn is_if_or_block(expr: &Expression) -> bool {
            if let Expression::IfElse { .. } = expr {
                return true;
            }

            return Parser::is_block(expr);
        }

        let condition = self.expression()?;
        let on_if = self.expression()?;

        let on_else = if let Lexeme::Keyword(crate::lexer::Keyword::Else) = self.lexeme {
            self.advance()?;
            self.expression()?
        }
        else {
            Expression::Null
        };

        if !Self::is_block(&on_if) { return Err(ErrorType::ParseError("Expected block after `if`".to_string())); }
        if !is_if_or_block(&on_else) { return Err(ErrorType::ParseError("Expected block after `else`".to_string())); }

        Ok(Expression::IfElse { condition: Box::new(condition), on_if: Box::new(on_if), on_else: Box::new(on_else) })
    }

    fn while_loop(&mut self) -> MudResult<Expression> {
        // assume `while` has already been consumed

        let condition = self.expression()?;
        let body = self.expression()?;

        dbg!(&body);

        if !Self::is_block(&body) { return Err(ErrorType::ParseError("Expected block after `while`".to_string())); }

        Ok(Expression::While { condition: Box::new(condition), body: Box::new(body) })
    }

    fn function(&mut self) -> MudResult<Expression> {
        fn is_decl(expr: &Expression) -> bool {
            if let Expression::BinaryOperation { op, lhs, .. } = expr {
                if let Expression::Identifier(_) = **lhs {
                    return *op == Operator::Colon;
                }
            }

            return false;
        }

        // assume `function` has already been consumed

        let mut args = Vec::new();

        expect_lexeme!(self, Lexeme::Operator(Operator::OpenParenthesis));

        loop {
            if let Lexeme::Operator(Operator::CloseParenthesis) = self.lexeme {
                break;
            }

            if args.len() != 0 {
                expect_lexeme!(self, Lexeme::Operator(Operator::Comma))
            }

            let arg = self.expression()?;
            if !is_decl(&arg) {
                return Err(ErrorType::ParseError("Malformed arguments in function type".to_string()));
            }

            args.push(arg)
        }

        self.advance()?;
        expect_lexeme!(self, Lexeme::Operator(Operator::Arrow));

        let return_type = Box::new(self.expression()?);
        let body = Box::new(self.expression()?);

        if !Self::is_block(&body) { return Err(ErrorType::ParseError("Expected block as function body".to_string())); }

        Ok(Expression::Function { args, return_type, body })
    }

    fn binary_operation(&mut self, precedence: u8) -> MudResult<Expression> {
        if precedence == 0 {
            return self.term();
        }

        let mut expr = self.binary_operation(precedence - 1)?;

        while let Lexeme::Operator(op) = self.lexeme {
            if let Some(&op_precedence) = PRECEDENCE_LOOKUP.get(&op) {
                if op_precedence == precedence {
                    self.advance()?;
                    expr = Expression::BinaryOperation { op, lhs: Box::new(expr), rhs: Box::new(self.binary_operation(precedence - 1)?) };
                } else {
                    break;
                }
            }
            else {
                break;
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
                Ok(Expression::UnaryOperation { op: Operator::Minus, oprand: Box::new(self.term()?) })
            }

            //print
            Lexeme::Operator(Operator::LessThan) => {
                Ok(Expression::UnaryOperation { op: Operator::LessThan, oprand: Box::new(self.term()?) })
            }

            Lexeme::Operator(Operator::OpenParenthesis) => {
                let expr = self.expression()?;

                if let Lexeme::Operator(Operator::CloseParenthesis) = self.lexeme {
                    self.advance()?;
                    Ok(expr)
                } else {
                    Err(ErrorType::ParseError("Unclosed parenthesis".to_string()))
                }
            }

            Lexeme::Operator(Operator::OpenBrace) => {
                let expr = self.expression()?;

                if let Lexeme::Operator(Operator::CloseBrace) = self.lexeme {
                    self.advance()?;
                    Ok(Expression::Block(Box::new(expr)))
                } else {
                    Err(ErrorType::ParseError("Unclosed brace".to_string()))
                }
            }

            Lexeme::Keyword(Keyword::If) => {
                self.ifelse()
            }

            Lexeme::Keyword(Keyword::While) => {
                self.while_loop()
            }

            Lexeme::Eof => Ok(Expression::Null),

            t => Err(ErrorType::ParseError(format!(
                "Expected term, recieved {:?}",
                t
            ))),
        }
    }

    fn advance(&mut self) -> MudResult<Lexeme> {
        Ok(std::mem::replace(&mut self.lexeme, self.lexer.next()?))
    }
}
