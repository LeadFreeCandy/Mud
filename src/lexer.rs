use std::collections::{HashMap, hash_map::Entry};

pub mod error;
use error::{MudResult, ErrorType};
// use crate::errors::{ParseResult, ErrorType};

use once_cell::sync::Lazy; // TODO: figure out why it cannot be unsync

static OPERATORS: Lazy<HashMap<&'static str, Operator>> = Lazy::new(|| {
    let mut operator_map: HashMap<&'static str, Operator> = HashMap::new();
    // let mut operators = [false; 256];

    operator_map.insert("+", Operator::Plus);
    operator_map.insert("-", Operator::Minus);
    operator_map.insert("*", Operator::Asterisk);
    operator_map.insert("(", Operator::OpenParenthesis);
    operator_map.insert(")", Operator::CloseParenthesis);
    operator_map.insert("{", Operator::OpenBrace);
    operator_map.insert("}", Operator::CloseBrace);

    operator_map.insert("<", Operator::LessThan);
    operator_map.insert(">", Operator::GreaterThan);

    operator_map.insert(";", Operator::Semicolon);
    operator_map.insert(":", Operator::Colon);
    operator_map.insert("=", Operator::Equals);
    operator_map
});

static KEYWORDS: Lazy<HashMap<&'static str, Keyword>> = Lazy::new(|| {
    let mut keyword_map: HashMap<&'static str, Keyword> = HashMap::new();
    // let mut operators = [false; 256];

    keyword_map.insert("if", Keyword::If);
    keyword_map.insert("else", Keyword::Else);

    keyword_map
});

static OP_CHARS: Lazy<[bool; 256]> = Lazy::new(||{

    let mut operators = [false; 256];
    for op in OPERATORS.keys() {
        for c in op.bytes() {
            operators[c as usize] = true;
        }
    }

    operators
});


#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Operator {
    //ops
    Plus,
    Minus,
    Asterisk,
    LessThan,
    GreaterThan,

    //assignment
    Equals,
    Colon,

    //delimiters
    Semicolon,
    OpenParenthesis,
    CloseParenthesis,

    OpenBrace,
    CloseBrace,
}

#[derive(Debug, Clone, Copy)]
pub enum Keyword {
    If,
    Else,
}

#[derive(Debug)]
pub enum Lexeme {
    Integer(u64),
    Identifier(String),
    Operator(Operator),
    Keyword(Keyword),
    Eof,
}

pub struct Lexer {
    program: Vec<u8>,
    index: usize,
}

impl Lexer {
    pub fn new(program: Vec<u8>) -> Self {
        Self {
            program,
            index: 0,
         }
    }

    pub fn next(&mut self) -> MudResult<Lexeme> {
        while self.peek().is_ascii_whitespace() {
            self.index += 1;
        }

        match self.peek() {
            c if c.is_ascii_digit() => self.integer(),
            c if c.is_ascii_alphabetic() => self.identifier(),
            c if OP_CHARS[c as usize] => self.operator(),
            0 => Ok(Lexeme::Eof),
            _ => Err(ErrorType::LexError("Invalid character".to_string()))
        }
    }

    fn integer(&mut self) -> MudResult<Lexeme> {
        let mut int: u64 = 0;

        while self.peek().is_ascii_digit() {
            int = int
                .checked_mul(10).ok_or(ErrorType::LexError("Overflowing integer literal".to_string()))?
                .checked_add((self.peek() - b'0') as u64).ok_or(ErrorType::LexError("Overflowing integer literal".to_string()))?;

            self.index += 1;
        }

        Ok(Lexeme::Integer(int))
    }

    fn identifier(&mut self) -> MudResult<Lexeme> {
        let start = self.index;

        while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
            self.index += 1;
        }

        match std::str::from_utf8(&self.program[start..self.index]) {
            Ok(v) => match KEYWORDS.get(v) {
                Some(k) => Ok(Lexeme::Keyword(*k)),
                None => Ok(Lexeme::Identifier(v.to_string()))
            }
            _ => Err(ErrorType::LexError("Identifier contained invalid bytes".to_string())),
        }
    }

    fn operator(&mut self) -> MudResult<Lexeme> {
        let mut op_string = String::new();

        let mut largest_op = None;
        let mut op_last_index = 0;

        while OP_CHARS[self.peek() as usize] {
            op_string.push(self.peek() as char);

            if let Some(op) = (*OPERATORS).get(&op_string as &str) {
                largest_op = Some(*op);
                op_last_index = self.index;
            }

            self.index += 1;
        }

        self.index = op_last_index + 1;

        Ok(Lexeme::Operator(largest_op.ok_or(ErrorType::LexError(format!("Invalid operator {}", op_string)))?))
    }

    fn peek(&mut self) -> u8 {
        *self.program.get(self.index).unwrap_or(&0)
    }
}
