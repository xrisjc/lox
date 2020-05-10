use std::error::Error;
use std::fmt;
use std::mem;
use std::rc::Rc;

use crate::chunk::Chunk;
use crate::op::*;
use crate::scanner::TokenTag::*;
use crate::scanner::{Scanner, Token, TokenTag};
use crate::value::Value;

use Precedence::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment,
    //Or,
    //And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    //Call,
    //Primary,
}

fn precedence_of(token: &Token) -> Precedence {
    use Precedence::*;

    match token.tag {
        Minus | Plus => Term,
        Slash | Star => Factor,
        BangEqual | EqualEqual => Equality,
        Greater | GreaterEqual | Less | LessEqual => Comparison,
        _ => None,
    }
}

#[derive(Debug)]
struct ParseError {
    token: Rc<Token>,
    message: String,
}

fn parse_error<T>(token: &Rc<Token>, message: &str) -> Result<T, ParseError> {
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

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Rc<Token>,
    previous: Rc<Token>,
}

impl<'a> Parser<'a> {
    fn new(source: &str) -> Parser {
        let token = Token {
            tag: Eof,
            lexeme: String::from(""),
            line: 0,
        };
        let token = Rc::new(token);

        Parser {
            scanner: Scanner::new(source),
            current: Rc::clone(&token),
            previous: Rc::clone(&token),
        }
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        let token = self.scanner.next_token();
        let token = Rc::new(token);
        if token.tag == Error {
            parse_error(&token, "error advancing")
        } else {
            self.previous = mem::replace(&mut self.current, token);
            Ok(())
        }
    }

    fn check(&mut self, tag: TokenTag) -> bool {
        self.current.tag == tag
    }

    fn matches(&mut self, tag: TokenTag) -> Result<bool, ParseError> {
        if !self.check(tag) {
            return Ok(false);
        }
        self.advance()?;
        return Ok(true);
    }

    fn consume(&mut self, tag: TokenTag, msg: &str) -> Result<(), ParseError> {
        if self.current.tag == tag {
            self.advance()
        } else {
            parse_error(&self.current, msg)
        }
    }

    fn prefix_rule(&mut self, chunk: &mut Chunk, can_assign: bool) -> Result<(), ParseError> {
        match self.previous.tag {
            False => {
                chunk.emit(OP_FALSE, self.previous.line);
            }
            Nil => {
                chunk.emit(OP_NIL, self.previous.line);
            }
            True => {
                chunk.emit(OP_TRUE, self.previous.line);
            }
            Identifier => {
                let token = Rc::clone(&self.previous);
                self.named_variable(chunk, &token, can_assign)?;
            }
            StringLiteral => {
                // The string is in the lexeme. We need to trim the leading and
                // trailing quotes.
                let s = &self.previous.lexeme;
                let s = &s[0..s.len()];
                let s = Value::new_string(s);

                chunk
                    .emit_constant(s, self.previous.line)
                    .or_else(|e| parse_error(&self.previous, &e))?;
            }
            Number => {
                let x: f64 = self
                    .previous
                    .lexeme
                    .parse()
                    .or_else(|_| parse_error(&self.previous, "Cannot parse number"))?;

                let x = Value::Number(x);

                chunk
                    .emit_constant(x, self.previous.line)
                    .or_else(|e| parse_error(&self.previous, &e))?;
            }
            LeftParen => {
                self.parse(Assignment, chunk)?;
                self.consume(RightParen, "Expect ')' after expression.")?;
            }
            Minus => {
                self.parse(Factor, chunk)?;
                chunk.emit(OP_NEGATE, self.previous.line);
            }
            Bang => {
                self.parse(Factor, chunk)?;
                chunk.emit(OP_NOT, self.previous.line);
            }
            _ => {
                parse_error(&self.previous, "unexpected token")?;
            }
        }

        Ok(())
    }

    fn infix_rule(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        let line = self.previous.line;

        match self.previous.tag {
            BangEqual => {
                self.parse(Equality, chunk)?;
                chunk.emit(OP_EQUAL, line);
                chunk.emit(OP_NOT, line);
            }
            Equal => {
                self.parse(Equality, chunk)?;
                chunk.emit(OP_EQUAL, line);
            }
            EqualEqual => {
                self.parse(Equality, chunk)?;
                chunk.emit(OP_EQUAL, line);
            }
            Greater => {
                self.parse(Comparison, chunk)?;
                chunk.emit(OP_GREATER, line);
            }
            GreaterEqual => {
                self.parse(Comparison, chunk)?;
                chunk.emit(OP_LESS, line);
                chunk.emit(OP_NOT, line);
            }
            Less => {
                self.parse(Comparison, chunk)?;
                chunk.emit(OP_LESS, line);
            }
            LessEqual => {
                self.parse(Comparison, chunk)?;
                chunk.emit(OP_GREATER, line);
                chunk.emit(OP_NOT, line);
            }
            Plus => {
                self.parse(Factor, chunk)?;
                chunk.emit(OP_ADD, line);
            }
            Minus => {
                self.parse(Factor, chunk)?;
                chunk.emit(OP_SUBTRACT, line);
            }
            Star => {
                self.parse(Unary, chunk)?;
                chunk.emit(OP_MULTIPLY, line);
            }
            Slash => {
                self.parse(Unary, chunk)?;
                chunk.emit(OP_DIVIDE, line);
            }
            _ => {
                parse_error(&self.previous, "expected operator")?;
            }
        }

        Ok(())
    }

    fn parse(&mut self, precedence: Precedence, chunk: &mut Chunk) -> Result<(), ParseError> {
        self.advance()?;

        let can_assign = precedence <= Precedence::Assignment;
        self.prefix_rule(chunk, can_assign)?;

        while precedence <= precedence_of(&self.current) {
            self.advance()?;
            self.infix_rule(chunk)?;
        }

        if can_assign && self.matches(Equal)? {
            return parse_error(&self.previous, "Invalid assignment target.");
        }

        Ok(())
    }

    fn expression(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        self.parse(Precedence::Assignment, chunk)
    }

    fn declaration(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        if self.matches(Var)? {
            self.var_declaration(chunk)
        } else {
            self.statement(chunk)
        }
    }

    fn parse_variable(&mut self, chunk: &mut Chunk, error_message: &str) -> Result<u8, ParseError> {
        self.consume(Identifier, error_message)?;
        identifier_constant(chunk, &self.previous)
    }

    fn named_variable(&mut self, chunk: &mut Chunk, token: &Rc<Token>, can_assign: bool) -> Result<(), ParseError> {
        let arg = identifier_constant(chunk, token)?;

        if can_assign && self.matches(Equal)? {
            self.expression(chunk)?;
            chunk.emit(OP_SET_GLOBAL, token.line);
            chunk.emit(arg, token.line);
        } else {
            chunk.emit(OP_GET_GLOBAL, token.line);
            chunk.emit(arg, token.line);
        }

        Ok(())
    }

    fn var_declaration(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        let global = self.parse_variable(chunk, "Expected variable name")?;

        let line = self.previous.line;

        if self.matches(Equal)? {
            self.expression(chunk)?;
        } else {
            chunk.emit(OP_NIL, line);
        }

        self.consume(Semicolon, "Expected ';' after variable declaration.")?;

        define_variable(chunk, line, global);

        Ok(())
    }

    fn synchronize(&mut self) {
        while self.current.tag != Eof {
            if self.previous.tag == Semicolon {
                return;
            }

            match self.current.tag {
                Class | Fun | Var | For | If | While | Print | Return => {
                    return;
                }
                _ => {
                    // Do nothing.
                }
            }

            // Ignore errors while syncing.
            let _e = self.advance();
        }
    }

    fn statement(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        if self.matches(Print)? {
            self.print_statement(chunk)
        } else {
            self.expression_statement(chunk)
        }
    }

    fn print_statement(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        let line = self.previous.line;

        self.expression(chunk)?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        chunk.emit(OP_PRINT, line);

        Ok(())
    }

    fn expression_statement(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        let line = self.previous.line;

        self.expression(chunk)?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        chunk.emit(OP_POP, line);

        Ok(())
    }
}

fn define_variable(chunk: &mut Chunk, line: usize, global: u8) {
    chunk.emit(OP_DEFINE_GLOBAL, line);
    chunk.emit(global, line);
}

fn identifier_constant(chunk: &mut Chunk, token: &Rc<Token>) -> Result<u8, ParseError> {
    let constant = Value::new_string(&token.lexeme);
    chunk
        .add_constant(constant)
        .or_else(|e| parse_error(token, &e))
}

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    let mut ok = true;

    let mut parser = Parser::new(source);
    if let Err(e) = parser.advance() {
        ok = false;
        eprintln!("{}", e);
    }
    loop {
        match parser.matches(Eof) {
            Ok(false) => {
                if let Err(e) = parser.declaration(chunk) {
                    ok = false;
                    eprintln!("{}", e);
                    parser.synchronize();
                }
            }
            Ok(true) => break,
            Err(e) => {
                ok = false;
                eprintln!("{}", e);
            }
        }
    }
    chunk.emit(OP_RETURN, parser.previous.line);

    if ok {
        chunk.disassemble("code");
    }

    return ok;
}
