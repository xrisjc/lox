use std::error::Error;
use std::fmt;
use std::rc::Rc;

use crate::scanner::{Token, TokenTag};

#[derive(Debug)]
pub struct ParseError {
    token: Rc<Token>,
    message: String,
}

pub fn parse_error<T>(token: &Rc<Token>, message: &str) -> Result<T, ParseError> {
    let token = Rc::clone(token);
    let message = String::from(message);
    Err(ParseError { token, message })
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] Error", self.token.line)?;

        match self.token.tag {
            TokenTag::Eof => write!(f, " at end")?,
            TokenTag::Error => {}
            _ => write!(f, " at '{}'", self.token.lexeme)?,
        }

        write!(f, ": {}", self.message)?;

        Ok(())
    }
}