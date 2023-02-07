use std::collections::HashMap;
use std::ops::Add;

use crate::parser::*;
use crate::lexer::error::{MudResult, ErrorType};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ValueType {
    Integer,
    Void,
    Unknown,
}

impl ValueType {
    fn from_string(str: &str) -> MudResult<Self> {
        match str {
            "int" => Ok(ValueType::Integer),
            _ => Err(ErrorType::CompileError("Invalid type".to_string()))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ExprType {
    Literal,
    Identifier,
    Expression,
}

#[derive(Clone)]
pub struct  Type {
    value: ValueType,
    expr: ExprType,
}

struct CompiledAtom {
    source: String,
    atom_type: Type,
}

pub struct Compiler {
    globals: HashMap<String, ValueType>,
}

impl CompiledAtom {
    fn new(source: String, value: ValueType, expr: ExprType) -> Self {
        Self {
            source,
            atom_type: Type { value, expr },
        }
    }
}

//     fn add(self, rhs: CompiledAtom) -> MudResult<Self> {
//         match (self.atom_type.value, rhs.atom_type.value) {
//             (ValueType::Integer, ValueType::Integer) => Ok(self.transform_source(format!("({}+{})", self.source, rhs.source))),
//             (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
//         }
//     }

//     fn sub(self, rhs: CompiledAtom) -> MudResult<Self> {
//         match (self.atom_type.value, rhs.atom_type.value) {
//             (ValueType::Integer, ValueType::Integer) => Ok(self.transform_source(format!("({}-{})", self.source, rhs.source))),
//             (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot subtract types {:?} and {:?}", l, r))),
//         }
//     }

//     fn mul(self, rhs: CompiledAtom) -> MudResult<Self> {
//         match (self.atom_type.value, rhs.atom_type.value) {
//             (ValueType::Integer, ValueType::Integer) => Ok(self.transform_source(format!("({}*{})", self.source, rhs.source))),
//             (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot multiply types {:?} and {:?}", l, r))),
//         }
//     }


//     fn div(self, rhs: CompiledAtom) -> MudResult<Self> {
//         match (self.atom_type.value, rhs.atom_type.value) {
//             (ValueType::Integer, ValueType::Integer) => Ok(self.transform_source(format!("({}/{})", self.source, rhs.source))),
//             (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot divide types {:?} and {:?}", l, r))),
//         }
//     }

//     fn comp(self, rhs: CompiledAtom) -> MudResult<Self> {
//         Ok(CompiledAtom::new(format!("{};\n{}", self.source, rhs.source), ValueType::Void, ExprType::Expression))
//     }

//     fn decl(self, rhs: CompiledAtom) -> MudResult<Self> {
//         if self.atom_type.value != ValueType::Unknown {
//             return MudResult::Err(ErrorType::CompileError(format!("Cannot redeclare {:?}", self.source)));
//         }

//         match (self.atom_type.expr, rhs.atom_type.expr) {
//             (ExprType::Identifier, ExprType::Identifier) => Ok(CompiledAtom::new(format!("{} {}", rhs.source, self.source), ValueType::Void, ExprType::Expression)),
//             (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot declare between types {:?} and {:?}", l, r))),
//         }
//     }

//     fn assign(self, rhs: CompiledAtom) -> MudResult<Self> {
//         match self.atom_type.expr {
//             ExprType::Identifier => {
//                 Ok(CompiledAtom::new(format!("{} = {}", self.source, rhs.source), ValueType::Void, ExprType::Expression))
//             }
//             e => MudResult::Err(ErrorType::CompileError(format!("Invalid lhs of assignment {:?}", e))),
//         }
//     }

//     fn negate(self) -> MudResult<Self> {
//         match self.atom_type.value {
//             ValueType::Integer => Ok(self.transform_source(format!("-({})", self.source))),
//             e => MudResult::Err(ErrorType::CompileError(format!("Cannot negate type {:?}", e))),
//         }
//     }

//     fn print(self) -> MudResult<Self> {
//         match self.atom_type.value {
//             _ => Ok(self.transform_source(format!("printf(\"%d\", {})", self.source))),
//             // e => MudResult::Err(ErrorType::CompileError(format!("Cannot print type {:?}", e))),
//         }
//     }

//     fn transform_source(&self, source: String) -> Self {
//         CompiledAtom::new(source, self.atom_type.value.clone(), ExprType::Expression)
//     }
// }

macro_rules! program_fmt {
    () => ("#include <stdio.h>\n\
            #include <stdlib.h>\n\
            int main() {{\n\
                {};\n\
            }}");
}

impl Compiler {
    pub fn new() -> Self {
        Self { globals: HashMap::new() }
    }

    pub fn compile_full(&mut self, program: Vec<u8>) -> MudResult<Vec<u8>>{
        let output = self.compile(program)?;

        Ok(format!(program_fmt!(), String::from_utf8(output).unwrap()).into_bytes())
    }

    pub fn compile(&mut self, program: Vec<u8>) -> MudResult<Vec<u8>> {
        let mut parser = Parser::new(program);
        let expression = parser.parse()?;

        let output = self.convert(expression)?.source.as_bytes().to_owned();
        Ok(output)
    }

    fn binary_op_transpile(&mut self, op: Operator, lhs: Expression, rhs: Expression) -> MudResult<CompiledAtom> {
        let lhs = self.convert(lhs)?;
        let rhs = self.convert(rhs)?;

        match op {
            Operator::Plus => self.add(lhs, rhs),
            Operator::Minus => self.sub(lhs, rhs),
            Operator::Asterisk => self.mul(lhs, rhs),
            Operator::Semicolon => self.comp(lhs, rhs),
            Operator::Colon => self.decl(lhs, rhs),
            Operator::Equals=> self.assign(lhs, rhs),

            _ => Err(ErrorType::CompileError(format!("Binary operator {:?} cannot be transpiled", op))),
        }
    }

    fn unary_op_transpile(&mut self, op: Operator, oprand: Expression) -> MudResult<CompiledAtom> {
        let oprand = self.convert(oprand)?;

        match op {
            Operator::Minus => self.negate(oprand),
            Operator::LessThan => self.print(oprand),
            _ => Err(ErrorType::CompileError(format!("Unary operator {:?} cannot be transpiled", op))),
        }
    }

    fn convert(&mut self, expression: Expression) -> MudResult<CompiledAtom> {
        match expression {
            Expression::Integer(val) => {
                Ok(CompiledAtom::new(val.to_string(), ValueType::Integer, ExprType::Literal))
            }
            Expression::Identifier(s) => {
                Ok(CompiledAtom::new(s, ValueType::Unknown, ExprType::Identifier))
            }
            Expression::UnaryOperation(op, expr) => {
                self.unary_op_transpile(op, *expr)
            }
            Expression::BinaryOperation(op, lhs, rhs) => {
                self.binary_op_transpile(op, *lhs, *rhs)
            }
            Expression::Null => Ok(CompiledAtom::new(String::new(), ValueType::Void, ExprType::Literal)),
        }
    }

    fn add(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (self.resolve_type(&lhs)?, self.resolve_type(&rhs)?) {
            (ValueType::Integer, ValueType::Integer) => Ok(CompiledAtom::new(format!("({}+{})", lhs.source, rhs.source), ValueType::Integer, ExprType::Expression)),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
        }
    }

    fn sub(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (self.resolve_type(&lhs)?, self.resolve_type(&rhs)?) {
            (ValueType::Integer, ValueType::Integer) => Ok(CompiledAtom::new(format!("({}-{})", lhs.source, rhs.source), ValueType::Integer, ExprType::Expression)),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot subtract types {:?} and {:?}", l, r))),
        }
    }

    fn mul(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (self.resolve_type(&lhs)?, self.resolve_type(&rhs)?) {
            (ValueType::Integer, ValueType::Integer) => Ok(CompiledAtom::new(format!("({}*{})", lhs.source, rhs.source), ValueType::Integer, ExprType::Expression)),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot multiply types {:?} and {:?}", l, r))),
        }
    }

    fn div(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (self.resolve_type(&lhs)?, self.resolve_type(&rhs)?) {
            (ValueType::Integer, ValueType::Integer) => Ok(CompiledAtom::new(format!("({}/{})", lhs.source, rhs.source), ValueType::Integer, ExprType::Expression)),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot divide types {:?} and {:?}", l, r))),
        }
    }

    fn comp(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        Ok(CompiledAtom::new(format!("{};\n{}", lhs.source, rhs.source), ValueType::Void, ExprType::Expression))
    }

    fn decl(&mut self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (lhs.atom_type.expr, rhs.atom_type.expr) {
            (ExprType::Identifier, ExprType::Identifier) => {
                let res = CompiledAtom::new(format!("{} {}", rhs.source, lhs.source), ValueType::Void, ExprType::Expression);

                if self.globals.insert(lhs.source, ValueType::from_string(&rhs.source)?).is_some() {
                    return MudResult::Err(ErrorType::CompileError("Variable redelcaration".to_string()));
                }

                Ok(res)
            }
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot declare between types {:?} and {:?}", l, r))),
        }
    }

    fn assign(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match lhs.atom_type.expr {
            ExprType::Identifier => {
                if *self.globals.get(&lhs.source).ok_or(ErrorType::CompileError("Undefined variable".to_string()))? != rhs.atom_type.value {
                    return MudResult::Err(ErrorType::CompileError("Wrong type".to_string()));
                }

                Ok(CompiledAtom::new(format!("{} = {}", lhs.source, rhs.source), ValueType::Void, ExprType::Expression))
            }
            e => MudResult::Err(ErrorType::CompileError(format!("Invalid lhs of assignment {:?}", e))),
        }
    }

    fn negate(&self, oprand: CompiledAtom) -> MudResult<CompiledAtom> {
        match self.resolve_type(&oprand)? {
            ValueType::Integer => Ok(CompiledAtom::new(format!("(-{})", oprand.source), ValueType::Integer, ExprType::Expression)),
            e => MudResult::Err(ErrorType::CompileError(format!("Cannot negate type {:?}", e))),
        }
    }

    fn print(&self, oprand: CompiledAtom) -> MudResult<CompiledAtom> {
        match self.resolve_type(&oprand)? {
            ValueType::Integer => Ok(CompiledAtom::new(format!("printf(\"%d\", {})", oprand.source), ValueType::Void, ExprType::Expression)),
            e => MudResult::Err(ErrorType::CompileError(format!("Cannot negate type {:?}", e))),
        }
    }

    fn resolve_type(&self, atom: &CompiledAtom) -> MudResult<ValueType> {
        match atom.atom_type.expr {
            ExprType::Identifier => self.globals.get(&atom.source).ok_or(ErrorType::CompileError("Undefined variable".to_string())).copied(),
            _ => Ok(atom.atom_type.value),
        }
    }
}

// pub fn compile(program: Vec<u8>) -> MudResult<Vec<u8>>{
//     let mut parser = Parser::new(program);
//     let expression = parser.parse()?;

//     let output = convert(expression)?.source.as_bytes().to_owned();
//     Ok(output)
// }

// pub fn compile_full(program: Vec<u8>) -> MudResult<Vec<u8>>{
//     let output = compile(program)?;

//     Ok(
//         format!(program_fmt!(), String::from_utf8(output).unwrap()).into_bytes()
//       )
// }

// fn binary_op_transpile(op: Operator, lhs: Expression, rhs: Expression) -> MudResult<CompiledAtom> {
//     match op {
//         Operator::Plus => convert(lhs)?.add(convert(rhs)?),
//         Operator::Minus => convert(lhs)?.sub(convert(rhs)?),
//         Operator::Asterisk => convert(lhs)?.mul(convert(rhs)?),
//         Operator::Semicolon=> convert(lhs)?.comp(convert(rhs)?),
//         Operator::Colon=> convert(lhs)?.decl(convert(rhs)?),
//         Operator::Equals=> convert(lhs)?.assign(convert(rhs)?),

//         _ => Err(ErrorType::CompileError(format!("Binary operator {:?} cannot be transpiled", op))),
//     }
// }

// fn unary_op_transpile(op: Operator, oprand: Expression) -> MudResult<CompiledAtom> {
//     match op {
//         Operator::Minus => convert(oprand)?.negate(),
//         Operator::LessThan => convert(oprand)?.print(),
//         _ => Err(ErrorType::CompileError(format!("Unary operator {:?} cannot be transpiled", op))),
//     }
// }

// fn convert(expression: Expression) -> MudResult<CompiledAtom> {
//     match expression {
//         Expression::Integer(val) => {
//             Ok(CompiledAtom::new(val.to_string(), ValueType::Integer, ExprType::Literal))
//         }
//         Expression::Identifier(s) => {
//             Ok(CompiledAtom::new(s, ValueType::Unknown, ExprType::Identifier))
//         }
//         Expression::UnaryOperation(op, expr) => {
//             unary_op_transpile(op, *expr)
//         }
//         Expression::BinaryOperation(op, lhs, rhs) => {
//             binary_op_transpile(op, *lhs, *rhs)
//         }
//         Expression::Null => Ok(CompiledAtom::new(String::new(), ValueType::Void, ExprType::Literal)),
//     }
// }
