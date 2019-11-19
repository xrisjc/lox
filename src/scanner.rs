use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum Lexeme {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    StringLiteral(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error(String),
    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub lexeme: Lexeme,
    pub line: usize,
}


fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') ||
    (c >= 'A' && c <= 'Z') ||
    c == '_'
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}


pub struct Scanner<'a> {
    itr: Peekable<Chars<'a>>,
    current: Option<char>,
    next: Option<char>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        let mut scanner = Scanner {
            itr: source.chars().peekable(),
            current: None,
            next: None,
            line: 1,
        };
        scanner.advance();
        scanner
    }

    fn advance(&mut self) {
        self.current = self.itr.next();
        self.next = self.itr.peek().map(|&c| c);
    }

    fn make_token(&self, lexem: Lexeme) -> Token {
        Token { lexeme: lexem, line: self.line }
    }

    pub fn next_token(&mut self) -> Token {
        use Lexeme::*;

        // Skip whitespace and comments.
        loop {
            match self.current {
                Some(' ') | Some('\r') | Some('\t') => {
                    self.advance();
                }
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                }
                Some('/') if self.next.map_or(false, |c| c == '/') => {
                    while self.current.map_or(false, |c| c != '\n') {
                        self.advance();
                    }
                }
                _ => break,
            }
        }

        // Handle end of code.
        if self.current.is_none() {
            return self.make_token(Eof);
        }

        // Handle a string literal.
        if let Some('"') = self.current {
            let mut s = String::new();
            self.advance();

            while self.current.map_or(false, |c| c != '"') {
                let c = self.current.unwrap();
                s.push(c);
                if c == '\n' {
                    self.line += 1;
                }
                self.advance();
            }

            if self.current.is_none() {
                let msg = "unterminated string".to_owned();
                return self.make_token(Error(msg));
            }

            // Skip past the closing quote.
            self.advance();

            return self.make_token(StringLiteral(s));
        }

        // Handle identifiers and keywords.
        if self.current.map_or(false, |c| is_alpha(c)) {
            let mut s = String::new();
            while self.current.map_or(false, |c| is_alpha(c) || self.current.map_or(false, |c| is_digit(c))) {
                s.push(self.current.unwrap());
                self.advance();
            }

            let lexem = match s.as_ref() {
                "and" => And,
                "class" => Class,
                "else" => Else,
                "false" => False,
                "for" => For,
                "fun" => Fun,
                "if" => If,
                "nil" => Nil,
                "or" => Or,
                "print" => Print,
                "return" => Return,
                "super" => Super,
                "this" => This,
                "true" => True,
                "var" => Var,
                "while" => While,
                _ => Identifier(s),
            };

            return self.make_token(lexem);
        }

        // Handle a number literal.
        if self.current.map_or(false, |c| is_digit(c)) {
            let mut s = String::new();

            while self.current.map_or(false, |c| is_digit(c)) {
                s.push(self.current.unwrap());
                self.advance();
            }

            // Look for fractional part.
            if self.current.map_or(false, |c| c == '.') && self.next.map_or(false, |c| is_digit(c)) {
                s.push(self.current.unwrap());
                self.advance();
                
                while self.current.map_or(false, |c| is_digit(c)) {
                    s.push(self.current.unwrap());
                    self.advance();
                }
            }

            return match s.parse() {
                Ok(x) => self.make_token(Number(x)),
                Err(_) => {
                    let msg = format!("'{}' cannot be converted to a number", s);
                    self.make_token(Error(msg))
                }
            };
        }

        // Handle operators.
        let lexeme = match self.current.unwrap() {
            '!' if self.next.map_or(false, |c| c == '=') => {
                self.advance();
                BangEqual
            }
            '=' if self.next.map_or(false, |c| c == '=') => {
                self.advance();
                EqualEqual
            }
            '<' if self.next.map_or(false, |c| c == '=') => {
                self.advance();
                LessEqual
            }
            '>' if self.next.map_or(false, |c| c == '=') => {
                self.advance();
                GreaterEqual
            }
            '(' => LeftParen,
            ')' => RightParen,
            '{' => LeftBrace,
            '}' => RightBrace,
            ';' => Semicolon,
            ',' => Comma,
            '.' => Dot,
            '-' => Minus,
            '+' => Plus,
            '/' => Slash,
            '*' => Star,
            '!' => Bang,
            '=' => Equal,
            '<' => Less,
            '>' => Greater,
            _ => {
                let msg = format!("unexpected character '{:?}'", self.current);
                self.advance();
                return self.make_token(Error(msg));
            }
        };

        // Advance past the last character in the operator.
        self.advance();

        return self.make_token(lexeme);
    }
}
