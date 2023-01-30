use std::result;

pub type ParseResult<T> = result::Result<T, ErrorType>;

#[derive(Debug)]
pub enum ErrorType {
    ParseError(String),
    LexError(String),
}
