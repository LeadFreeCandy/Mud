use crate::parser::*;
use crate::lexer::error::{ParseResult, ErrorType};

pub fn compile(program: Vec<u8>) -> ParseResult<Vec<u8>>{
    let mut parser = Parser::new(program);
    let expression = parser.parse()?;

    let output = convert(expression).as_bytes().to_owned();
    Ok(output)
}

fn op_to_str(op: Operator) -> String{
    match op {
        Operator::Plus => "+",
        Operator::Minus => "-",
        Operator::Asterisk => "*",
        Operator::OpenParenthesis => "(",
        Operator::CloseParenthesis=> ")",
    }.to_string()
}

fn convert(expression: Expression) -> String{
    match expression {
        Expression::Integer(val) => {
            val.to_string()
        },
        Expression::UnaryOperation(op, expr) => {
            "(".to_string() + &op_to_str(op) + &convert(*expr) + ")"
        },
        Expression::BinaryOperation(op, expr1, expr2) => {
            "(".to_string() + &convert(*expr1) + &op_to_str(op) + &convert(*expr2) + ")"
        }
    }
}
