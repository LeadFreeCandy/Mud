use std::collections::HashMap;

use crate::parser::*;
use crate::lexer::error::{MudResult, ErrorType};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueType {
    I32,
    U8,
    Void,
    Pointer(Box<ValueType>),
    Unknown,
    Function { args: Vec<ValueType>, return_type: Box<ValueType> },
    Struct (HashMap<String, ValueType>),
}

#[derive(Debug, Clone)]
pub enum ExprType {
    Literal,
    FunctionLiteral { args: Vec<Expression>, return_type: Box<Expression>, body: Box<Expression> },
    StructLiteral {fields: Vec<Expression>},
    Identifier,
    Type,
    Expression,
}

#[derive(Clone, Debug)]
pub struct  Type {
    value: ValueType,
    expr: ExprType,
}

#[derive(Debug, Clone)]
struct CompiledAtom {
    source: String,
    atom_type: Type,
}

pub struct Compiler {
    scope_stack: Vec<HashMap<String, ValueType>>,
    forward_decls: String,
}

impl CompiledAtom {
    fn new(source: String, value: ValueType, expr: ExprType) -> Self {
        Self {
            source,
            atom_type: Type { value, expr },
        }
    }
}

macro_rules! program_fmt {
    () => ("#include <stdio.h>\n\
            #include <stdlib.h>\n\
            typedef int i32;\n\
            {}\n\
            {}");
}

impl Compiler {
    pub fn new() -> Self {
        let mut globals = HashMap::new();

        globals.insert("calloc".to_string(), ValueType::Function { args: vec![ValueType::I32], return_type: Box::new(ValueType::Pointer(Box::new(ValueType::Void))) });

        Self { scope_stack: vec![globals], forward_decls: String::new() }
    }

    pub fn compile_full(&mut self, program: Vec<u8>) -> MudResult<Vec<u8>>{
        let output = self.compile(program)?;
        assert!(self.scope_stack.len() == 1);

        Ok(format!(program_fmt!(), self.forward_decls, String::from_utf8(output).unwrap()).into_bytes())
    }

    pub fn compile(&mut self, program: Vec<u8>) -> MudResult<Vec<u8>> {
        let mut parser = Parser::new(program);
        let expression = parser.parse()?;

        let output = self.convert(expression)?.source.as_bytes().to_owned();
        Ok(output)
    }

    fn struct_assign(&mut self, lhs: CompiledAtom, rhs: Expression) -> MudResult<CompiledAtom>{
        todo!("Havent implemented this yet");
    }

    fn binary_op_transpile(&mut self, op: Operator, lhs: Expression, rhs: Expression) -> MudResult<CompiledAtom> {
        let lhs = self.convert(lhs)?;
        if let ExprType::Identifier = lhs.atom_type.expr {
            if let Expression::Block(inner) = rhs {
                return self.struct_assign(lhs, *inner);
            }
        }
        let rhs = self.convert(rhs)?;

        match op {
            Operator::Plus => self.add(lhs, rhs),
            Operator::Minus => self.sub(lhs, rhs),
            Operator::Asterisk => self.mul(lhs, rhs),
            Operator::Semicolon => self.comp(lhs, rhs),
            Operator::Colon => self.decl(lhs, rhs),
            Operator::Equals=> self.assign(lhs, rhs),
            Operator::ColonEquals=> self.assign_func_struct(lhs, rhs),

            _ => Err(ErrorType::CompileError(format!("Binary operator {:?} cannot be transpiled", op))),
        }
    }

    fn unary_op_transpile(&mut self, op: Operator, oprand: Expression) -> MudResult<CompiledAtom> {
        let oprand = self.convert(oprand)?;

        match oprand.atom_type.expr {
            ExprType::Type => {
                // unreachable!("should not be calling unary transpile this");
                match op {
                    Operator::Asterisk => self.pointer_type(oprand),
                    _ => Err(ErrorType::CompileError(format!("Unary operator {:?} on type cannot be transpiled", op))),
                }
            },
            _ => {
                match op {
                    Operator::Minus => self.negate(oprand),
                    Operator::LessThan => self.print(oprand),
                    Operator::Asterisk => self.deref(oprand),
                    Operator::Ampersand => self.adressof(oprand),
                    _ => Err(ErrorType::CompileError(format!("Unary operator {:?} cannot be transpiled", op))),
                }
            }
        }
    }

    fn block(&mut self, expression: Expression) -> MudResult<CompiledAtom> {
        self.scope_stack.push(HashMap::new());
        let atom = CompiledAtom { source: format!("{{\n{};\n}}", self.convert(expression)?.source), atom_type: Type { value: ValueType::Void, expr: ExprType::Expression } };
        self.scope_stack.pop();
        Ok(atom)
    }

    fn supressed_block(&mut self, expression: Expression) -> MudResult<CompiledAtom> {
        // scope has been created for us
        let atom = CompiledAtom { source: format!("{{\n{};\n}}", self.convert(expression)?.source), atom_type: Type { value: ValueType::Void, expr: ExprType::Expression } };
        self.scope_stack.pop();
        Ok(atom)
    }

    fn if_else(&mut self, condition: Expression, on_if: Expression, on_else: Expression) -> MudResult<CompiledAtom> {
        Ok(CompiledAtom {
            source: format!("if ({}) {} else {}", self.convert(condition)?.source, self.convert(on_if)?.source, self.convert(on_else)?.source),
            atom_type: Type { value: ValueType::Void, expr: ExprType::Expression
        }})
    }

    fn while_loop(&mut self, condition: Expression, body: Expression) -> MudResult<CompiledAtom> {
        Ok(CompiledAtom {
            source: format!("while ({}) {}", self.convert(condition)?.source, self.convert(body)?.source),
            atom_type: Type { value: ValueType::Void, expr: ExprType::Expression
        }})
    }

    fn function(&mut self, args: Vec<Expression>, return_type: Box<Expression>, body: Box<Expression>) -> MudResult<CompiledAtom> {
        Ok(CompiledAtom {
            source: "_mud_uncompiled_function".to_string(),
            atom_type: Type { value: ValueType::Unknown, expr: ExprType::FunctionLiteral { args, return_type, body } },
        })
    }

    fn r#struct(&mut self, fields: Vec<Expression>) -> MudResult<CompiledAtom>{
        Ok(CompiledAtom {
            source: "_mud_uncompiled_struct".to_string(),
            atom_type: Type { value: ValueType::Unknown, expr: ExprType::StructLiteral {fields}},
        })
    }

    fn function_call(&mut self, function: Expression, args: Vec<Expression>) -> MudResult<CompiledAtom> {
        let function = self.convert(function)?;

        let mut source = String::new();

        match self.resolve_type(&function)? { // NOTE: not type checking here at all
            ValueType::Function { args: _, return_type } => {
                source.push_str(&function.source);
                source.push('(');

                for arg in args {
                    source.push_str(&self.convert(arg)?.source);
                }

                source.push(')');

                Ok(CompiledAtom { source, atom_type: Type { expr: ExprType::Expression, value: *return_type } })
            },
            t => Err(ErrorType::CompileError(format!("Cannot call a {:?}", t)))
        }
    }

    fn return_statement(&mut self, value: Expression) -> MudResult<CompiledAtom> {
        Ok(CompiledAtom { source: format!("return {}", self.convert(value)?.source), atom_type: Type { value: ValueType::Unknown, expr: ExprType::Expression } })
    }

    fn convert(&mut self, expression: Expression) -> MudResult<CompiledAtom> {
        match expression {
            Expression::Integer(val) => {
                Ok(CompiledAtom::new(val.to_string(), ValueType::I32, ExprType::Literal))
            }
            Expression::Identifier(s) => {
                if s == "i32" { //todo fix this placeholder
                    Ok(CompiledAtom::new("int".to_string(), ValueType::Unknown, ExprType::Type))
                } else if s == "u8" {
                    Ok(CompiledAtom::new("char".to_string(), ValueType::Unknown, ExprType::Type))
                }
                else {
                    Ok(CompiledAtom::new(s, ValueType::Unknown, ExprType::Identifier))
                }
            }
            Expression::String(s) => {
                Ok(CompiledAtom::new("\"".to_string() + &s + &"\"", ValueType::Pointer(Box::new(ValueType::U8)), ExprType::Literal))
            }
            Expression::UnaryOperation { op, oprand: expr } => {
                self.unary_op_transpile(op, *expr)
            }
            Expression::BinaryOperation { op, lhs, rhs } => {
                self.binary_op_transpile(op, *lhs, *rhs)
            }
            Expression::Block(expr) => {
                self.block(*expr)
            }
            Expression::IfElse { condition, on_if, on_else } => {
                self.if_else(*condition, *on_if, *on_else)
            }
            Expression::While { condition, body } => {
                self.while_loop(*condition, *body)
            }
            Expression::Function { args, return_type, body } => {
                self.function(args, return_type, body)
            }
            Expression::Struct {fields} => {
                self.r#struct(fields)
            }
            Expression::FunctionCall { function, args } => {
                self.function_call(*function, args)
            }
            Expression::Return(value) => {
                self.return_statement(*value)
            }
            Expression::Null => Ok(CompiledAtom::new(String::new(), ValueType::Void, ExprType::Literal)),
        }
    }

    fn add(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (self.resolve_type(&lhs)?, self.resolve_type(&rhs)?) {
            (ValueType::I32, ValueType::I32) => Ok(CompiledAtom::new(format!("({}+{})", lhs.source, rhs.source), ValueType::I32, ExprType::Expression)),
            (ValueType::Pointer(inner), ValueType::I32) => Ok(CompiledAtom::new(format!("({}+{})", lhs.source, rhs.source), ValueType::Pointer(inner), ExprType::Expression)),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot add types {:?} and {:?}", l, r))),
        }
    }

    fn sub(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (self.resolve_type(&lhs)?, self.resolve_type(&rhs)?) {
            (ValueType::I32, ValueType::I32) => Ok(CompiledAtom::new(format!("({}-{})", lhs.source, rhs.source), ValueType::I32, ExprType::Expression)),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot subtract types {:?} and {:?}", l, r))),
        }
    }

    fn mul(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (self.resolve_type(&lhs)?, self.resolve_type(&rhs)?) {
            (ValueType::I32, ValueType::I32) => Ok(CompiledAtom::new(format!("({}*{})", lhs.source, rhs.source), ValueType::I32, ExprType::Expression)),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot multiply types {:?} and {:?}", l, r))),
        }
    }

    fn div(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (self.resolve_type(&lhs)?, self.resolve_type(&rhs)?) {
            (ValueType::I32, ValueType::I32) => Ok(CompiledAtom::new(format!("({}/{})", lhs.source, rhs.source), ValueType::I32, ExprType::Expression)),
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot divide types {:?} and {:?}", l, r))),
        }
    }

    fn comp(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        Ok(CompiledAtom::new(format!("{};\n{}", lhs.source, rhs.source), ValueType::Void, ExprType::Expression))
    }

    fn decl(&mut self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        match (lhs.atom_type.expr, &rhs.atom_type.expr) {
            (ExprType::Identifier, ExprType::Type) => {
                let res = CompiledAtom::new(format!("{} {}", rhs.source, lhs.source), ValueType::Void, ExprType::Expression);

                let rhs_type = self.find_type(&rhs);

                if self.scope_stack.last_mut().unwrap().insert(lhs.source, rhs_type?).is_some() {
                    return MudResult::Err(ErrorType::CompileError("Variable redelcaration".to_string()));
                }


                Ok(res)
            },
            (ExprType::Identifier, ExprType::Identifier) => {
                let res = CompiledAtom::new(format!("{} {}", rhs.source, lhs.source), ValueType::Void, ExprType::Expression);
                Ok(res)
            }
            (l, r) => MudResult::Err(ErrorType::CompileError(format!("Cannot declare between types {:?} and {:?}", l, r))),
        }
    }

    fn assign(&self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        fn ensure_types_equal(lhs: &ValueType, rhs: &ValueType) -> MudResult<()> {
            if *lhs == ValueType::U8 || *lhs == ValueType::I32 && *rhs == ValueType::U8 || *rhs == ValueType::I32 {
                return Ok(());
            }

            if let ValueType::Pointer(t) = rhs {
                if **t == ValueType::Void {
                    if let ValueType::Pointer(_) = lhs {
                        return Ok(());
                    }
                }
            }

            if lhs == rhs {
                return Ok(())
            }

            MudResult::Err(ErrorType::CompileError(format!("Expected type {lhs:?} but got type {rhs:?} in assignment")))
        }

        match lhs.atom_type.expr {
            ExprType::Identifier => {
                let lhs_type = self.resolve_type(&lhs)?;
                let rhs_type = rhs.atom_type.value;

                ensure_types_equal(&lhs_type, &rhs_type)?;

                Ok(CompiledAtom::new(format!("{} = {}", lhs.source, rhs.source), ValueType::Void, ExprType::Expression))
            }
            ExprType::Expression => {
                let lhs_type = self.resolve_type(&lhs)?;
                let rhs_type = rhs.atom_type.value;

                ensure_types_equal(&lhs_type, &rhs_type)?;

                Ok(CompiledAtom::new(format!("{} = {}", lhs.source, rhs.source), ValueType::Void, ExprType::Expression))
            }
            e => {
                MudResult::Err(ErrorType::CompileError(format!("Invalid lhs of assignment {:?}", e)))
            },
        }
    }

    fn assign_func_struct(&mut self, lhs: CompiledAtom, rhs: CompiledAtom) -> MudResult<CompiledAtom> {
        fn resolve_args(this: &mut Compiler, args: Vec<Expression>, scope: &mut HashMap<String, ValueType>) -> MudResult<(Vec<String>, Vec<ValueType>)> {
            let mut strs = Vec::new();
            let mut types = Vec::new();

            for arg in args {
                if let Expression::BinaryOperation { op, lhs, rhs } = arg {
                    let rhs = this.convert(*rhs)?;
                    if let (Operator::Colon, Expression::Identifier(ident), ExprType::Type) = (op, *lhs, &rhs.atom_type.expr) {
                        strs.push(format!("{} {ident}", &rhs.source));
                        types.push(this.find_type(&rhs)?);
                        scope.insert(ident, types.last().unwrap().clone());
                        continue;
                    }
                }

                return Err(ErrorType::CompileError("Malformed function arguments".to_string()));
            }

            Ok((strs, types))
        }

        match (lhs.atom_type.expr, rhs.atom_type.expr) {
            (ExprType::Identifier, ExprType::FunctionLiteral { args, return_type, body }) => {
                if dbg!(self.scope_stack.len()) != 1 {
                    return MudResult::Err(ErrorType::CompileError("Functions are not allowed outside the top level".to_string()));
                }

                let return_converted = self.convert(*return_type)?;
                let return_type_string = &return_converted.source;

                let mut fn_scope = HashMap::new();
                let (strs, types) = resolve_args(self, args, &mut fn_scope)?;
                let f_type = ValueType::Function { args: types, return_type: Box::new(self.find_type(&return_converted)?) };

                if self.scope_stack.last_mut().unwrap().insert(lhs.source.clone(), f_type).is_some() {
                    return MudResult::Err(ErrorType::CompileError("Function redelcaration".to_string()));
                }

                self.scope_stack.push(fn_scope);

                let header = format!("{} {}({})", return_type_string, lhs.source, strs.join(", "));
                self.forward_decls.push_str(&header);
                self.forward_decls.push_str(";\n");

                let result = Ok(CompiledAtom::new(format!("{}{}", header, self.convert(*body)?.source), ValueType::Void, ExprType::Expression));
                self.scope_stack.pop();
                result
            },
            (ExprType::Identifier, ExprType::StructLiteral{fields}) => {
                if dbg!(self.scope_stack.len()) != 1 {
                    return MudResult::Err(ErrorType::CompileError("Structs are not allowed outside the top level".to_string()));
                }

                let mut fn_scope = HashMap::new(); //this is a dummy scope
                let (strs, types) = resolve_args(self, fields, &mut fn_scope)?;

                let mut fields = HashMap::new();
                    for (str, ftype) in strs.iter().zip(types.into_iter()){
                        if fields.insert(str.to_owned(), ftype).is_some(){
                            return MudResult::Err(ErrorType::CompileError("Duplicate field in struct".to_string()));
                        }
                    }
                let s_type = ValueType::Struct(fields);

                if self.scope_stack.last_mut().unwrap().insert(lhs.source.clone(), s_type).is_some() {
                    return MudResult::Err(ErrorType::CompileError("Struct redelcaration".to_string()));
                }

                let result = Ok(CompiledAtom::new(
                        format!("typedef struct {{ {} }} {};",
                                strs.join("; ") + &";",
                                lhs.source),
                        ValueType::Void,
                        ExprType::Expression
                        ));
                result
            }
            e => MudResult::Err(ErrorType::CompileError(format!("Invalid lhs of assignment {:?}", e))),
        }
    }

    fn negate(&self, oprand: CompiledAtom) -> MudResult<CompiledAtom> {
        match self.resolve_type(&oprand)? {
            ValueType::I32 => Ok(CompiledAtom::new(format!("(-{})", oprand.source), ValueType::I32, ExprType::Expression)),
            e => MudResult::Err(ErrorType::CompileError(format!("Cannot negate type {:?}", e))),
        }
    }

    fn adressof(&self, oprand: CompiledAtom) -> MudResult<CompiledAtom> {
        //todo should probably check if you can actually wrap this thing but whatever
        Ok(CompiledAtom::new(format!("(&{})", oprand.source), ValueType::Pointer(Box::new(self.resolve_type(&oprand)?)), ExprType::Expression))
        // match self.resolve_type(&oprand)? {
        //     ValueType::Pointer(inner) => Ok(CompiledAtom::new(format!("(&{})", oprand.source), ValueType::Pointer(Box::new()), ExprType::Expression)),
        //     ValueType::Integer => Ok(CompiledAtom::new(format!("(&{})", oprand.source), ValueType::Integer, ExprType::Expression)),
        //     e => MudResult::Err(ErrorType::CompileError(format!("Cannot negate type {:?}", e))),
        // }
    }

    fn deref(&self, oprand: CompiledAtom) -> MudResult<CompiledAtom> {
        dbg!("called deref");
        match self.resolve_type(&oprand)? {
            ValueType::Pointer(inner) => Ok(CompiledAtom::new(format!("*{}", oprand.source), *inner, ExprType::Expression)),
            // ValueType::Integer => Ok(CompiledAtom::new(format!("(-{})", oprand.source), ValueType::Integer, ExprType::Expression)),
            e => MudResult::Err(ErrorType::CompileError(format!("Cannot deref type {:?}", e))),
        }
    }

    fn pointer_type(&self, oprand: CompiledAtom) -> MudResult<CompiledAtom> {
        dbg!(self.resolve_type(&oprand));
        dbg!(&oprand);
        Ok::<CompiledAtom, ErrorType>(CompiledAtom::new(format!("{}*", oprand.source), ValueType::Pointer(Box::new(self.resolve_type(&oprand)?)), ExprType::Type))
    }

    fn print(&self, oprand: CompiledAtom) -> MudResult<CompiledAtom> {
        match self.resolve_type(&oprand)? {
            ValueType::I32 => Ok(CompiledAtom::new(format!("printf(\"%d\", {})", oprand.source), ValueType::Void, ExprType::Expression)),
            ValueType::U8 => Ok(CompiledAtom::new(format!("printf(\"%c\", {})", oprand.source), ValueType::Void, ExprType::Expression)),
            //todo fix
            ValueType::Pointer(inner) => Ok(CompiledAtom::new(format!("printf(\"%s\", {})", oprand.source), ValueType::Void, ExprType::Expression)),
            // ValueType::StringLiteral => Ok(CompiledAtom::new(format!("printf(\"%s\", {})", oprand.source), ValueType::Void, ExprType::Expression)),
            e => MudResult::Err(ErrorType::CompileError(format!("Cannot print type {:?}", e))),
        }
    }


    fn find_type(&self, atom: &CompiledAtom) -> MudResult<ValueType> {
        match atom.atom_type.expr {
            ExprType::Type => {
                if let ValueType::Pointer(inner) = &atom.atom_type.value{
                    let source = atom.source[..atom.source.len()-1].to_string();
                    let mut atom_type = atom.atom_type.clone();
                    atom_type.value = *inner.clone();

                    let fake_atom = CompiledAtom{source, atom_type};

                    return Ok(ValueType::Pointer(Box::new(self.find_type(&fake_atom)?)));
                }
                match &atom.source[..] {
                    "int" => {
                        return Ok(ValueType::I32)
                    }
                    "char" => {
                        return Ok(ValueType::U8)
                    }
                    &_ => todo!("add more types")
                }
            }
            _ => Ok(atom.atom_type.value.clone()),
        }
    }

    fn resolve_type(&self, atom: &CompiledAtom) -> MudResult<ValueType> {
        match atom.atom_type.expr {
            ExprType::Identifier => {
                for scope in self.scope_stack.iter().rev() {
                    if let Some(v) = scope.get(&atom.source) {
                        return Ok(v.clone());
                    }
                }

                Err(ErrorType::CompileError(format!("Undefined variable: {}", atom.source)))
            }
            _ => Ok(atom.atom_type.value.clone()),
        }
    }
}
