use std::{collections::HashMap, env};

fn main() {
    let args = env::args();
    let args: Vec<String> = args.collect();
    if let Some(file_path) = args.get(1) {
        let code = std::fs::read_to_string(file_path).expect("Cant read file");
        run(code);
    } else {
        panic!("You need provide path to file");
    }
}

#[derive(Debug, Clone)]
enum Token {
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
    Identifier(String),
    String(String),
    Number(f64),

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

#[derive(Debug)]
struct TokenInfo {
    token: Token,
    line: usize,
}
struct Scanner {
    source: Vec<char>,
    tokens: Vec<TokenInfo>,
    start: usize,
    current: usize,
    line: usize,
    reserved_words: HashMap<String, Token>,
}

impl Scanner {
    fn new(source: String) -> Self {
        let mut reserved_words = HashMap::with_capacity(16);
        reserved_words.insert("and".to_string(), Token::And);
        reserved_words.insert("class".to_string(), Token::Class);
        reserved_words.insert("else".to_string(), Token::Else);
        reserved_words.insert("false".to_string(), Token::False);
        reserved_words.insert("fun".to_string(), Token::Fun);
        reserved_words.insert("for".to_string(), Token::For);
        reserved_words.insert("if".to_string(), Token::If);
        reserved_words.insert("nil".to_string(), Token::Nil);
        reserved_words.insert("or".to_string(), Token::Or);
        reserved_words.insert("print".to_string(), Token::Print);
        reserved_words.insert("return".to_string(), Token::Return);
        reserved_words.insert("super".to_string(), Token::Super);
        reserved_words.insert("this".to_string(), Token::This);
        reserved_words.insert("true".to_string(), Token::True);
        reserved_words.insert("var".to_string(), Token::Var);
        reserved_words.insert("while".to_string(), Token::While);
        println!("{}", reserved_words.len());
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::default(),
            start: 0,
            current: 0,
            line: 1,
            reserved_words,
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.add_token(Token::EOF);
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            c if c.is_whitespace() => {}
            '.' => self.add_token(Token::Dot),
            ',' => self.add_token(Token::Comma),
            ';' => self.add_token(Token::Semicolon),
            '+' => self.add_token(Token::Plus),
            '-' => self.add_token(Token::Minus),
            '*' => self.add_token(Token::Star),
            '(' => self.add_token(Token::LeftParen),
            ')' => self.add_token(Token::RightParen),
            '{' => self.add_token(Token::LeftBrace),
            '}' => self.add_token(Token::RightBrace),
            '!' => {
                if self.match_char('=') {
                    self.add_token(Token::BangEqual)
                } else {
                    self.add_token(Token::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(Token::EqualEqual)
                } else {
                    self.add_token(Token::Equal)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(Token::LessEqual)
                } else {
                    self.add_token(Token::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(Token::GreaterEqual)
                } else {
                    self.add_token(Token::Greater)
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != Some('\n') {
                        self.advance();
                    }
                } else {
                    self.add_token(Token::Slash)
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
            .cloned()
            .unwrap_or(Token::Identifier(identifier));
        self.add_token(token);
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
        let number: String = self.source[self.start..self.current].into_iter().collect();
        let number: f64 = number.parse().unwrap();
        self.add_token(Token::Number(number));
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
                    self.add_token(Token::String(s));
                    break;
                }
                _ => s.push(self.advance()),
            }
        }
    }
    fn add_token(&mut self, token: Token) {
        self.tokens.push(TokenInfo {
            token,
            line: self.line,
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

fn run(source: String) {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens();
    for token in scanner.tokens {
        println!("{:#?}", token)
    }
}
