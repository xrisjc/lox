use std::mem;

use crate::chunk::Chunk;
use crate::op::*;
use crate::scanner::{Lexeme, Token, Scanner};
use crate::scanner::Lexeme::*;
use crate::value::Value;

use Precedence::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

fn precedence_of(token: &Token) -> Precedence {
    use Precedence::*;
    
    match token.lexeme {
        Minus | Plus => Term,
        Slash | Star => Factor,
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
            current: Token { lexeme: Eof, line: 0 },
            previous: Token { lexeme: Eof, line: 0 },
            had_error: false,
            panic_mode: false,
        }
    }

    fn advance(&mut self) {
        loop {
            match self.scanner.next_token() {
                Token { lexeme: Error(_), .. } if self.panic_mode => {
                    // Don't report errors in panic mode.
                }
                Token { lexeme: Error(msg), line } => {
                    self.panic_mode = true;
                    self.had_error = true;
                    eprintln!("[line {}] Error: {}", line, msg);
                }
                token => {
                    self.previous = mem::replace(&mut self.current, token);
                    break;
                }
            }
        }
    }

    fn consume(&mut self, lexeme: Lexeme, msg: &str) {
        if self.current.lexeme == lexeme {
            self.advance();
        } else if self.panic_mode {
            // Don't report errors in panic mode.
        } else {
            self.panic_mode = true;
            self.had_error = true;
            eprintln!("[line {}] Error: {}", self.current.line, msg);
        }
    }

    fn prefix_rule(&mut self, chunk: &mut Chunk) {
        match self.previous {
            Token { lexeme: False, line } => {
                chunk.emit(OP_FALSE, line);
            }
            Token { lexeme: Nil, line } => {
                chunk.emit(OP_NIL, line);
            }
            Token { lexeme: True, line } => {
                chunk.emit(OP_TRUE, line);
            }
            Token { lexeme: Number(x), line } => {
                if !chunk.emit_constant(Value::Number(x), line) {
                    // TODO: Original called an error function here.
                    panic!("Too many constants in one chunk.");
                }
            }
            Token { lexeme: LeftParen, .. } => {
                self.parse(Assignment, chunk);
                self.consume(RightParen, "Expect ')' after expression.");
            }
            Token { lexeme: Minus, line } => {
                self.parse(Factor, chunk);
                chunk.emit(OP_NEGATE, line);
            }
            Token { lexeme: Bang, line } => {
                self.parse(Factor, chunk);
                chunk.emit(OP_NOT, line);
            }
            Token { ref lexeme, .. } => {
                // TODO: Original called an error function here.
                panic!("unexpected token {:?}", lexeme);
            }
        }
    }

    fn infix_rule(&mut self, chunk: &mut Chunk) {
        match self.previous {
            Token { lexeme: BangEqual, line } => {
                self.parse(Equality, chunk);
                chunk.emit(OP_EQUAL, line);
                chunk.emit(OP_NOT, line);
            }
            Token { lexeme: Equal, line } => {
                self.parse(Equality, chunk);
                chunk.emit(OP_EQUAL, line);
            }
            Token { lexeme: Greater, line } => {
                self.parse(Comparison, chunk);
                chunk.emit(OP_GREATER, line);
            }
            Token { lexeme: GreaterEqual, line } => {
                self.parse(Comparison, chunk);
                chunk.emit(OP_LESS, line);
                chunk.emit(OP_NOT, line);
            }
            Token { lexeme: Less, line } => {
                self.parse(Comparison, chunk);
                chunk.emit(OP_LESS, line);
            }
            Token { lexeme: LessEqual, line } => {
                self.parse(Comparison, chunk);
                chunk.emit(OP_GREATER, line);
                chunk.emit(OP_NOT, line);
            }
            Token { lexeme: Plus, line } => {
                self.parse(Factor, chunk);
                chunk.emit(OP_ADD, line);
            }
            Token { lexeme: Minus, line } => {
                self.parse(Factor, chunk);
                chunk.emit(OP_SUBTRACT, line);
            }
            Token { lexeme: Star, line } => {
                self.parse(Unary, chunk);
                chunk.emit(OP_MULTIPLY, line);
            }
            Token { lexeme: Slash, line } => {
                self.parse(Unary, chunk);
                chunk.emit(OP_DIVIDE, line);
            }
            _ => {
                // TODO: This is supposedly unreachable.
                panic!("Expected expression");
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

