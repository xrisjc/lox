use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenTag {
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
    Identifier,
    StringLiteral,
    Number,

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

    Error,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tag: TokenTag,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn error(&self, message: &str) {
        eprint!("[line {}] Error", self.line);

        match self.tag {
            TokenTag::Eof => eprint!(" at end"),
            TokenTag::Error => { },
            _ => eprint!(" at '{}'", self.lexeme),
        }

        eprintln!(": {}", message);
    }
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

    fn make_token(&self, tag: TokenTag, lexeme: String) -> Token {
        Token { tag: tag, lexeme: lexeme, line: self.line }
    }

    fn make_token_str(&self, tag: TokenTag, lexeme: &str) -> Token {
        self.make_token(tag, String::from(lexeme))
    }

    pub fn next_token(&mut self) -> Token {
        use TokenTag::*;

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
            return self.make_token_str(Eof, "");
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
                return self.make_token_str(Error, "unterminated string");
            }

            // Skip past the closing quote.
            self.advance();

            return self.make_token(StringLiteral, s);
        }

        // Handle identifiers and keywords.
        if self.current.map_or(false, |c| is_alpha(c)) {
            let mut s = String::new();
            while self.current.map_or(false, |c| is_alpha(c) || self.current.map_or(false, |c| is_digit(c))) {
                s.push(self.current.unwrap());
                self.advance();
            }

            let tag = match s.as_ref() {
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
                _ => Identifier,
            };

            return self.make_token(tag, s);
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

            return self.make_token(Number, s);
        }

        // Handle operators.
        let token = match self.current.unwrap() {
            '!' if self.next.map_or(false, |c| c == '=') => {
                self.advance();
                self.make_token_str(BangEqual, "!=")
            }
            '=' if self.next.map_or(false, |c| c == '=') => {
                self.advance();
                self.make_token_str(EqualEqual, "==")
            }
            '<' if self.next.map_or(false, |c| c == '=') => {
                self.advance();
                self.make_token_str(LessEqual, "<=")
            }
            '>' if self.next.map_or(false, |c| c == '=') => {
                self.advance();
                self.make_token_str(GreaterEqual, ">=")
            }
            '(' => self.make_token_str(LeftParen, "("),
            ')' => self.make_token_str(RightParen, ")"),
            '{' => self.make_token_str(LeftBrace, "{"),
            '}' => self.make_token_str(RightBrace, "}"),
            ';' => self.make_token_str(Semicolon, ";"),
            ',' => self.make_token_str(Comma, ","),
            '.' => self.make_token_str(Dot, "."),
            '-' => self.make_token_str(Minus, "-"),
            '+' => self.make_token_str(Plus, "+"),
            '/' => self.make_token_str(Slash, "/"),
            '*' => self.make_token_str(Star, "*"),
            '!' => self.make_token_str(Bang, "!"),
            '=' => self.make_token_str(Equal, "="),
            '<' => self.make_token_str(Less, "<"),
            '>' => self.make_token_str(Greater, ">"),
            _ => {
                let msg = format!("unexpected character '{:?}'", self.current);
                self.advance();
                self.make_token(Error, msg)
            }
        };

        // Advance past the last character in the operator.
        self.advance();

        return token;
    }
}
