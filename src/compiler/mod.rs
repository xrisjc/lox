#![allow(dead_code)]

mod error;
mod locals;

use std::mem;
use std::rc::Rc;


use crate::chunk::Chunk;
use crate::op::*;
use crate::scanner::TokenTag::*;
use crate::scanner::{Scanner, Token, TokenTag};
use crate::value::Value;

use Precedence::*;
use locals::Local;
use error::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Base,
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
        _ => Base,
    }
}

type ParseResult = Result<(), ParseError>;

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Rc<Token>,
    previous: Rc<Token>,
    locals: Vec<Local>,
    scope_depth: i32,
}

const MAX_LOCALS: usize = 255;

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
            locals: Vec::with_capacity(MAX_LOCALS),
            scope_depth: 0,
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, chunk: &mut Chunk) {
        self.scope_depth -= 1;

        while self.locals.len() > 0 && self.locals[self.locals.len()-1].depth > self.scope_depth {
            chunk.emit(OP_POP, self.previous.line);
            self.locals.pop();
        }
    }

    fn advance(&mut self) -> ParseResult {
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

    fn consume(&mut self, tag: TokenTag, msg: &str) -> ParseResult {
        if self.current.tag == tag {
            self.advance()
        } else {
            parse_error(&self.current, msg)
        }
    }

    fn prefix_rule(&mut self, chunk: &mut Chunk, can_assign: bool) -> ParseResult {
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

    fn infix_rule(&mut self, chunk: &mut Chunk) -> ParseResult {
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

    fn parse(&mut self, precedence: Precedence, chunk: &mut Chunk) -> ParseResult {
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

    fn expression(&mut self, chunk: &mut Chunk) -> ParseResult {
        self.parse(Precedence::Assignment, chunk)
    }

    fn declaration(&mut self, chunk: &mut Chunk) -> ParseResult {
        if self.matches(Var)? {
            self.var_declaration(chunk)
        } else {
            self.statement(chunk)
        }
    }

    fn named_variable(&mut self, chunk: &mut Chunk, token: &Rc<Token>, can_assign: bool) -> ParseResult {

        let kind = if let Some(arg) = self.resolve_local(token)? {
            (arg, OP_GET_LOCAL, OP_SET_LOCAL)
        } else {
            let arg = identifier_constant(chunk, token)?;
            (arg, OP_GET_GLOBAL, OP_SET_GLOBAL)
        };

        let (arg, get_op, set_op) = kind;

        if can_assign && self.matches(Equal)? {
            self.expression(chunk)?;
            chunk.emit(set_op, token.line);
            chunk.emit(arg, token.line);
        } else {
            chunk.emit(get_op, token.line);
            chunk.emit(arg, token.line);
        }

        Ok(())
    }

    fn var_declaration(&mut self, chunk: &mut Chunk) -> ParseResult {
        let global = self.parse_variable(chunk, "Expected variable name")?;

        let line = self.previous.line;

        if self.matches(Equal)? {
            self.expression(chunk)?;
        } else {
            chunk.emit(OP_NIL, line);
        }

        self.consume(Semicolon, "Expected ';' after variable declaration.")?;

        self.define_variable(chunk, line, global);

        Ok(())
    }

    fn define_variable(&mut self, chunk: &mut Chunk, line: usize, global: u8) {
        if self.scope_depth == 0 {
            chunk.emit(OP_DEFINE_GLOBAL, line);
            chunk.emit(global, line);
        } else if self.scope_depth > 0 {
            self.mark_initialized();
        }
    }

    fn parse_variable(&mut self, chunk: &mut Chunk, error_message: &str) -> Result<u8, ParseError> {
        self.consume(Identifier, error_message)?;

        self.declare_variable()?;
        if self.scope_depth > 0 {
            Ok(0)
        } else {
            identifier_constant(chunk, &self.previous)
        }
    }

    fn mark_initialized(&mut self) {
        let last_offset = self.locals.len() - 1;
        self.locals[last_offset].depth = self.scope_depth;
    }

    fn declare_variable(&mut self) -> ParseResult {
        if self.scope_depth > 0 {
            let name = Rc::clone(&self.previous);
            self.add_local(&name)?;
        }

        Ok(())
    }

    fn add_local(&mut self, name: &Rc<Token>) -> ParseResult {
        if self.locals.len() >= MAX_LOCALS {
            return parse_error(name, "Exceeded maximum number of local variables.");
        }

        for local in self.locals.iter().rev() {
            if local.depth == -1 && local.depth < self.scope_depth {
                break;
            }

            if name.lexeme == local.name.lexeme {
                return parse_error(name, "Variable with this name already declared in this scope.");
            }
        }

        let local = Local::new(name);
        self.locals.push(local);
        Ok(())
    }

    fn resolve_local(&mut self, name: &Rc<Token>) -> Result<Option<u8>, ParseError> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name.lexeme == name.lexeme {
                if local.depth == -1 {
                    return parse_error(name, "Cannot read local variable in its own initializer.");
                }
                return Ok(Some(i as u8));
            }
        }

        return Ok(None);
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

    fn statement(&mut self, chunk: &mut Chunk) -> ParseResult {
        if self.matches(Print)? {
            self.print_statement(chunk)
        } else if self.matches(If)? {
            self.if_statement(chunk)
        } else if self.matches(LeftBrace)? {
            self.begin_scope();
            self.block(chunk)?;
            self.end_scope(chunk);
            Ok(())
        } else {
            self.expression_statement(chunk)
        }
    }

    fn block(&mut self, chunk: &mut Chunk) -> ParseResult {
        while !self.check(RightBrace) && !self.check(Eof) {
            self.declaration(chunk)?;
        }
        self.consume(RightBrace, "Expected '}' after block.")?;
        Ok(())
    }

    fn print_statement(&mut self, chunk: &mut Chunk) -> ParseResult {
        let line = self.previous.line;

        self.expression(chunk)?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        chunk.emit(OP_PRINT, line);

        Ok(())
    }

    fn expression_statement(&mut self, chunk: &mut Chunk) -> ParseResult {
        let line = self.previous.line;

        self.expression(chunk)?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        chunk.emit(OP_POP, line);

        Ok(())
    }

    fn if_statement(&mut self, chunk: &mut Chunk) -> ParseResult {
        let if_token = Rc::clone(&self.previous);
        let line = self.previous.line;

        self.consume(LeftParen, "Expect '(' after 'if'.")?;
        self.expression(chunk)?;
        self.consume(RightParen, "Expect ')' after condition.")?;

        let then_jump = chunk.emit_jump(OP_JUMP_IF_FALSE, line);
        let line = self.current.line;
        chunk.emit(OP_POP, line);
        self.statement(chunk)?;

        let line = self.current.line;
        let else_jump = chunk.emit_jump(OP_JUMP, line);
        
        chunk
            .patch_jump(then_jump)
            .or_else(|e| parse_error(&if_token, &e))?;

        chunk.emit(OP_POP, line);

        if self.matches(Else)? {
            self.statement(chunk)?;
        }

        chunk
            .patch_jump(else_jump)
            .or_else(|e| parse_error(&if_token, &e))?;

        Ok(())
    }
}

/// Adds the token's lexeme to the chunk's constant table.  Returns the index
/// in the constant table.
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
