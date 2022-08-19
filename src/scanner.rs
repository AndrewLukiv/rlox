use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens
    Dot,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Star,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Slash,

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
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
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

    EOF,
}

#[derive(Debug,Clone)]
pub struct TokenInfo {
    pub token_type: TokenType,
    pub line: usize,
    pub lexeme: String,
    pub number: Option<f64>,
}

pub struct Scanner {
    source: Vec<char>,
    pub tokens: Vec<TokenInfo>,
    start: usize,
    current: usize,
    line: usize,
    reserved_words: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: &String) -> Self {
        let mut reserved_words = HashMap::with_capacity(16);
        reserved_words.insert("and".to_string(), TokenType::And);
        reserved_words.insert("class".to_string(), TokenType::Class);
        reserved_words.insert("else".to_string(), TokenType::Else);
        reserved_words.insert("false".to_string(), TokenType::False);
        reserved_words.insert("fun".to_string(), TokenType::Fun);
        reserved_words.insert("for".to_string(), TokenType::For);
        reserved_words.insert("if".to_string(), TokenType::If);
        reserved_words.insert("nil".to_string(), TokenType::Nil);
        reserved_words.insert("or".to_string(), TokenType::Or);
        reserved_words.insert("print".to_string(), TokenType::Print);
        reserved_words.insert("return".to_string(), TokenType::Return);
        reserved_words.insert("super".to_string(), TokenType::Super);
        reserved_words.insert("this".to_string(), TokenType::This);
        reserved_words.insert("true".to_string(), TokenType::True);
        reserved_words.insert("var".to_string(), TokenType::Var);
        reserved_words.insert("while".to_string(), TokenType::While);
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::default(),
            start: 0,
            current: 0,
            line: 1,
            reserved_words,
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.add_token(TokenType::EOF,"");
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            c if c.is_whitespace() => {}
            '.' => self.add_token(TokenType::Dot, '.'),
            ',' => self.add_token(TokenType::Comma, ','),
            ';' => self.add_token(TokenType::Semicolon, ';'),
            '+' => self.add_token(TokenType::Plus, '+'),
            '-' => self.add_token(TokenType::Minus, '-'),
            '*' => self.add_token(TokenType::Star, '*'),
            '(' => self.add_token(TokenType::LeftParen, '('),
            ')' => self.add_token(TokenType::RightParen, ')'),
            '{' => self.add_token(TokenType::LeftBrace, '{'),
            '}' => self.add_token(TokenType::RightBrace, '}'),
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual,"!=")
                } else {
                    self.add_token(TokenType::Bang,'!')
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual,"==")
                } else {
                    self.add_token(TokenType::Equal,'=')
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual,"<=")
                } else {
                    self.add_token(TokenType::Less,'<')
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual,">=")
                } else {
                    self.add_token(TokenType::Greater,'>')
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != Some('\n') {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash,'/')
                }
            }
            '"' => self.string(),
            c if Self::is_digit(c) => self.number(),
            c if c.is_ascii_alphabetic() => self.identifier(),
            c => println!("Unexpected character {:?}", c),
        }
    }

    fn is_digit(c: char) -> bool {
        ('0'..='9').contains(&c)
    }

    fn identifier(&mut self) {
        loop {
            let is_alphanumeric = if let Some(c) = self.peek() {
                c.is_ascii_alphanumeric()
            } else {
                false
            };
            if is_alphanumeric {
                self.advance();
            } else {
                break;
            }
        }
        let identifier: String = self.source[self.start..self.current].into_iter().collect();
        let token = self
            .reserved_words
            .get(&identifier)
            .cloned() ;
        if let Some(token_type) = token{
            self.add_token(token_type,identifier);
        }else{
            self.add_token(TokenType::Identifier,identifier)
        };
    }

    fn number(&mut self) {
        loop {
            let next_character = self.peek();
            let after_next_is_digit = if let Some(c) = self.peek_next() {
                Self::is_digit(c)
            } else {
                false
            };
            match next_character {
                Some(c) if Self::is_digit(c) || (c == '.' && after_next_is_digit) => self.advance(),
                None | Some(_) => break,
            };
        }
        let number_str: String = self.source[self.start..self.current].into_iter().collect();
        let number: f64 = number_str.parse().unwrap();
        self.add_number_token(number_str,number);
    }

    fn string(&mut self) {
        let mut s = String::new();
        loop {
            match self.peek() {
                None => {
                    panic!(
                        "Unterminated string with value {:?} at line {}",
                        s, self.line
                    );
                }
                Some('"') => {
                    self.advance();
                    self.add_token(TokenType::String,s);
                    break;
                }
                _ => s.push(self.advance()),
            }
        }
    }
    fn add_number_token(&mut self, lexeme: String, number: f64) {
        self.tokens.push(TokenInfo {
            token_type: TokenType::Number,
            line: self.line,
            lexeme,
            number: Some(number),
        });
    }
    fn add_token(&mut self, token: TokenType, lexeme: impl std::fmt::Display) {
        self.tokens.push(TokenInfo {
            token_type: token,
            line: self.line,
            lexeme: lexeme.to_string(),
            number: None,
        });
    }
    fn current_char(&self) -> char {
        self.source[self.current]
    }

    fn advance(&mut self) -> char {
        let c = self.current_char();
        if c == '\n' {
            self.line += 1
        }
        self.current += 1;
        c
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.current_char() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.current_char())
        }
    }
    fn peek_next(&self) -> Option<char> {
        let next_index = self.current + 1;
        if next_index >= self.source.len() {
            None
        } else {
            Some(self.source[next_index])
        }
    }
}
