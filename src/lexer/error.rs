use std::result;

pub type MudResult<T> = result::Result<T, ErrorType>;

#[derive(Debug)]
pub enum ErrorType {
    ParseError(String),
    LexError(String),
    CompileError(String),
}
