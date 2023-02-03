use std::ops::Add;

use crate::parser::*;
use crate::lexer::error::{MudResult, ErrorType};

#[derive(Debug)]
pub enum Type {
    Integer,
    Void,
}

#[derive(Debug)]
pub enum CompilerType {
    Type(Type),
    Identifier,
}

struct CompiledAtom {
    source: String,
    expr_type: CompilerType,
}

impl CompiledAtom {
    fn new(source: String, expr_type: CompilerType) -> Self {
        Self {
            source,
            expr_type,
        }
    }

    fn add(self, rhs: CompiledAtom) -> MudResult<Self> {
        match (self.expr_type, rhs.expr_type) {
            (CompilerType::Type(Type::Integer), CompilerType::Type(Type::Integer)) => Ok(self.transform_source(format!("({}+{})", self.source, rhs.source))),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
        }
    }

    fn sub(self, rhs: CompiledAtom) -> MudResult<Self> {
        match (self.expr_type, rhs.expr_type) {
            (CompilerType::Type(Type::Integer), CompilerType::Type(Type::Integer)) => Ok(self.transform_source(format!("({}-{})", self.source, rhs.source))),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
        }
    }

    fn mul(self, rhs: CompiledAtom) -> MudResult<Self> {
        match (self.expr_type, rhs.expr_type) {
            (CompilerType::Type(Type::Integer), CompilerType::Type(Type::Integer)) => Ok(self.transform_source(format!("({}*{})", self.source, rhs.source))),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
        }
    }


    fn div(self, rhs: CompiledAtom) -> MudResult<Self> {
        match (self.expr_type, rhs.expr_type) {
            (CompilerType::Type(Type::Integer), CompilerType::Type(Type::Integer)) => Ok(self.transform_source(format!("({}/{})", self.source, rhs.source))),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
        }
    }

    fn comp(self, rhs: CompiledAtom) -> MudResult<Self> {
        Ok(CompiledAtom::new(format!("{};{}", self.source, rhs.source), CompilerType::Type(Type::Void)))
    }

    fn decl(self, rhs: CompiledAtom) -> MudResult<Self> {
        match (self.expr_type, rhs.expr_type) {
            (CompilerType::Identifier, CompilerType::Identifier) => Ok(CompiledAtom::new(format!("{} {}", self.source, rhs.source), CompilerType::Type(Type::Void))),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
        }
    }

    fn assign(self, rhs: CompiledAtom) -> MudResult<Self> {
        match (self.expr_type, rhs.expr_type) {
            (CompilerType::Identifier, e) => Ok(CompiledAtom::new(format!("{} {}", self.source, rhs.source), CompilerType::Type(Type::Void))),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
        }
    }

    fn negate(self) -> MudResult<Self> {
        match self.expr_type {
            CompilerType::Type(Type::Integer) =>
        }
    }

    fn transform_source(self, source: String) -> Self {
        CompiledAtom::new(source, self.expr_type)
    }
}

pub fn compile(program: Vec<u8>) -> MudResult<Vec<u8>>{
    let mut parser = Parser::new(program);
    let expression = parser.parse()?;

    let output = convert(expression)?.source.as_bytes().to_owned();
    Ok(output)
}

pub fn compile_full(program: Vec<u8>) -> MudResult<Vec<u8>>{
    let output = compile(program)?;

    Ok(
        format!(
"#include <stdio.h>
#include <stdlib.h>

int main() {{
    {}}}",
            String::from_utf8(output).unwrap()
            ).into_bytes()
      )
}

fn binary_op_transpile(op: Operator, lhs: Expression, rhs: Expression) -> MudResult<CompiledAtom> {
    match op {
        Operator::Plus => convert(lhs)?.add(convert(rhs)?),
        Operator::Minus => convert(lhs)?.sub(convert(rhs)?),
        Operator::Asterisk => convert(lhs)?.mul(convert(rhs)?),
        Operator::Semicolon=> convert(lhs)?.comp(convert(rhs)?),
        Operator::Colon=> convert(lhs)?.decl(convert(rhs)?),
        Operator::Equals=> convert(lhs)?.assign(convert(rhs)?),

        _ => Err(ErrorType::CompileError(format!("Binary operator {:?} cannot be transpiled", op))),
    }
}

fn unary_op_transpile(op: Operator, oprand: Expression) -> MudResult<String> {
    match op {
        Operator::Minus => Ok(format!("-({})", convert(oprand)?)),
        Operator::LessThan => Ok(format!("printf(\"%ll\", {})", convert(oprand)?)),
        _ => Err(ErrorType::CompileError(format!("Unary operator {:?} cannot be transpiled", op))),
    }
}

fn convert(expression: Expression) -> MudResult<CompiledAtom> {
    match expression {
        Expression::Integer(val) => {
            Ok(val.to_string())
        },
        Expression::Identifier(s) => {
            Ok(s)n
        }
        Expression::UnaryOperation(op, expr) => {
            unary_op_transpile(op, &convert(*expr)?)
        },
        Expression::BinaryOperation(op, lhs, rhs) => {
            binary_op_transpile(op, &convert(*lhs)?, &convert(*rhs)?)
        },
        Expression::Null => Ok(String::new()),
    }
}
