use std::mem;

use crate::chunk::Chunk;
use crate::object::{Obj, ObjValue};
use crate::op::*;
use crate::scanner::{TokenTag, Token, Scanner};
use crate::scanner::TokenTag::*;
use crate::value::Value;

use Precedence::*;
use std::rc::Rc;

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

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &str) -> Parser {
        Parser {
            scanner: Scanner::new(source),
            current: Token { tag: Eof, lexeme: String::from(""), line: 0 },
            previous: Token { tag: Eof, lexeme: String::from(""), line: 0 },
            had_error: false,
            panic_mode: false,
        }
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        token.error(message);
        self.had_error = true;
    }

    fn advance(&mut self) {
        loop {
            let token = self.scanner.next_token();
            match token.tag {
                Error if self.panic_mode => {
                    // Don't report errors in panic mode.
                }
                Error => {
                    self.panic_mode = true;
                    self.error_at(&token, "error advancing");
                }
                _ => {
                    self.previous = mem::replace(&mut self.current, token);
                    break;
                }
            }
        }
    }

    fn consume(&mut self, tag: TokenTag, msg: &str) {
        if self.current.tag == tag {
            self.advance();
        } else if self.panic_mode {
            // Don't report errors in panic mode.
        } else {
            self.panic_mode = true;
            self.error_at(&self.current.clone(), msg);
        }
    }

    fn prefix_rule(&mut self, chunk: &mut Chunk) {
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
            StringLiteral => {
                // The string is in the lexeme. We need to trim the leading and
                // trailing quotes.
                let s = &self.previous.lexeme;
                let s = &s[0..s.len()];
                
                // Create a string and wrap it up in an Obj.
                let s = String::from(s);
                let s = ObjValue::String(s);
                let s = Obj { value: s };
                let s = Rc::new(s);
                let s = Value::Obj(s);

                if !chunk.emit_constant(s, self.previous.line) {
                    self.error_at(&self.previous.clone(), "Too many constants in one chunk.");
                }
            }
            Number => {
                let x = match self.previous.lexeme.parse() {
                    Ok(x) => Value::Number(x),
                    Err(_) => {
                        self.error_at(&self.previous.clone(), "cannot be converted to a number");
                        return;
                    }
                };

                if !chunk.emit_constant(x, self.previous.line) {
                    self.error_at(&self.previous.clone(), "Too many constants in one chunk.");
                }
            }
            LeftParen => {
                self.parse(Assignment, chunk);
                self.consume(RightParen, "Expect ')' after expression.");
            }
            Minus => {
                self.parse(Factor, chunk);
                chunk.emit(OP_NEGATE, self.previous.line);
            }
            Bang => {
                self.parse(Factor, chunk);
                chunk.emit(OP_NOT, self.previous.line);
            }
            _ => {
                self.panic_mode = true;
                self.error_at(&self.previous.clone(), "unexpected token");
            }
        }
    }

    fn infix_rule(&mut self, chunk: &mut Chunk) {
        let line = self.previous.line;

        match self.previous.tag {
            BangEqual => {
                self.parse(Equality, chunk);
                chunk.emit(OP_EQUAL, line);
                chunk.emit(OP_NOT, line);
            }
            Equal => {
                self.parse(Equality, chunk);
                chunk.emit(OP_EQUAL, line);
            }
            EqualEqual => {
                self.parse(Equality, chunk);
                chunk.emit(OP_EQUAL, line);
            }
            Greater => {
                self.parse(Comparison, chunk);
                chunk.emit(OP_GREATER, line);
            }
            GreaterEqual => {
                self.parse(Comparison, chunk);
                chunk.emit(OP_LESS, line);
                chunk.emit(OP_NOT, line);
            }
            Less => {
                self.parse(Comparison, chunk);
                chunk.emit(OP_LESS, line);
            }
            LessEqual => {
                self.parse(Comparison, chunk);
                chunk.emit(OP_GREATER, line);
                chunk.emit(OP_NOT, line);
            }
            Plus => {
                self.parse(Factor, chunk);
                chunk.emit(OP_ADD, line);
            }
            Minus => {
                self.parse(Factor, chunk);
                chunk.emit(OP_SUBTRACT, line);
            }
            Star => {
                self.parse(Unary, chunk);
                chunk.emit(OP_MULTIPLY, line);
            }
            Slash => {
                self.parse(Unary, chunk);
                chunk.emit(OP_DIVIDE, line);
            }
            _ => {
                self.error_at(&self.previous.clone(), "expected operator");
            }
        }
    }

    fn parse(&mut self, precedence: Precedence, chunk: &mut Chunk) {
        self.advance();
        self.prefix_rule(chunk);

        while precedence <= precedence_of(&self.current) {
            self.advance();
            self.infix_rule(chunk);
        }
    }
}

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    let mut parser = Parser::new(source);
    parser.advance();
    parser.parse(Precedence::Assignment, chunk);
    parser.consume(Eof, "Expect end of expression");
    chunk.emit(OP_RETURN, parser.previous.line);

    if !parser.had_error {
        chunk.disassemble("code");
    }

    return !parser.had_error;
}

