use std::{char, iter::Peekable, str::CharIndices};

use crate::token::{TType, Token};

#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    token_start: usize,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.char_indices().peekable();
        Self {
            source,
            token_start: chars.peek().map(|(index, _c)| *index).unwrap_or_default(),
            chars,
            line: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next().map(|(_index, char)| char)
    }

    fn match_char(&mut self, expected: char) -> bool {
        match self.chars.peek() {
            Some((_index, char)) => {
                if *char == expected {
                    let _ = self.advance();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn match_str(&mut self, expected: &str) -> bool {
        let strlen = expected.len();
        let byte = match self.chars.peek() {
            Some((index, _char)) => *index,
            None => return false,
        };
        let offset = byte + strlen;
        if offset > self.source.len() {
            return false;
        }
        if expected == &self.source[byte..offset] {
            // consume character if true
            for _char in 0..expected.chars().count() {
                let _ = self.advance();
            }
            return true;
        }
        false
    }

    fn current(&mut self) -> usize {
        self.chars
            .peek()
            .map(|(index, _)| *index)
            .unwrap_or(self.source.len())
    }

    fn content(&mut self) -> &'a str {
        let len = self.current();
        &self.source[self.token_start..len]
    }

    fn make_token(&mut self, ttype: TType) -> Token<'a> {
        Token::new(ttype, Some(self.content()), self.line)
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.chars.peek() {
                Some((_index, ' ')) | Some((_index, '\t')) | Some((_index, '\r')) => {
                    self.advance();
                }
                Some((_index, '\n')) => {
                    self.line += 1;
                    self.advance();
                }
                Some((_index, '/')) => {
                    if self.match_str("//") {
                        while let Some((_index, char)) = self.chars.peek() {
                            if *char == '\n' {
                                break;
                            } else {
                                self.advance();
                            }
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        self.token_start = self.current();
        // let next = self.chars.peek().map(|(_, char)| *char);

        let c = self.advance();
        if is_identifier(&c) {
            return self.identifier();
        }
        if is_digit(&c) {
            return self.number_literal();
        }

        match c {
            None => Token::new(TType::Eof, None, self.line),
            Some(char) => match char {
                '(' => self.make_token(TType::LeftParen),
                ')' => self.make_token(TType::RightParen),
                '{' => self.make_token(TType::LeftBrace),
                '}' => self.make_token(TType::RightBrace),
                ';' => self.make_token(TType::SemiColon),
                ',' => self.make_token(TType::Comma),
                '.' => self.make_token(TType::Dot),
                '+' => self.make_token(TType::Plus),
                '-' => self.make_token(TType::Minus),
                '/' => self.make_token(TType::Slash),
                '*' => self.make_token(TType::Star),
                '!' => {
                    if self.match_char('=') {
                        self.make_token(TType::BangEqual)
                    } else {
                        self.make_token(TType::Bang)
                    }
                }
                '=' => {
                    if self.match_char('=') {
                        self.make_token(TType::EqualEqual)
                    } else {
                        self.make_token(TType::Equal)
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        self.make_token(TType::LessEqual)
                    } else {
                        self.make_token(TType::Less)
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        self.make_token(TType::GreaterEqual)
                    } else {
                        self.make_token(TType::Greater)
                    }
                }
                '"' => self.string_literal(),
                _ => self.make_token(TType::UnexpectedCharacterError),
            },
        }
    }

    fn identifier(&mut self) -> Token<'a> {
        while match self.chars.peek() {
            Some((_index, char)) => is_identifier(&Some(*char)),
            None => false,
        } {
            self.advance();
        }

        let tt = self.identifier_type();
        self.make_token(tt)
    }

    fn identifier_type(&mut self) -> TType {
        let word = self.content();
        if word.is_empty() {
            return TType::Identifer;
        }
        match &word[..1] {
            "a" => check_key(word, "and", 1, TType::And),
            "c" => check_key(word, "class", 1, TType::Class),
            "e" => check_key(word, "else", 1, TType::Else),
            "f" => {
                if word.len() < 2 {
                    TType::Identifer
                } else {
                    match &word[1..2] {
                        "a" => check_key(word, "false", 2, TType::False),
                        "o" => check_key(word, "for", 2, TType::For),
                        "u" => check_key(word, "fun", 2, TType::Fun),
                        _ => TType::Identifer,
                    }
                }
            }
            "i" => check_key(word, "if", 1, TType::If),
            "n" => check_key(word, "nil", 1, TType::Nil),
            "o" => check_key(word, "or", 1, TType::Or),
            "p" => check_key(word, "print", 1, TType::Print),
            "r" => check_key(word, "return", 1, TType::Return),
            "s" => check_key(word, "super", 1, TType::Super),
            "t" => {
                if word.len() < 2 {
                    TType::Identifer
                } else {
                    match &word[1..2] {
                        "h" => check_key(word, "this", 2, TType::This),
                        "r" => check_key(word, "true", 2, TType::True),
                        _ => TType::Identifer,
                    }
                }
            }
            "v" => check_key(word, "var", 1, TType::Var),
            "w" => check_key(word, "while", 1, TType::While),
            _ => TType::Identifer,
        }
    }

    fn string_literal(&mut self) -> Token<'a> {
        loop {
            match self.chars.peek() {
                Some((_index, '"')) => {
                    self.advance();
                    return self.make_token(TType::String);
                }
                Some((_index, '\n')) => {
                    self.advance();
                    self.line += 1;
                }
                _ => return self.make_token(TType::UnterminatedStringError),
            }
        }
    }

    fn consume_number(&mut self) {
        while match self.chars.peek() {
            Some((_index, char)) => is_digit(&Some(*char)),
            None => false,
        } {
            self.advance();
        }
    }

    fn number_literal(&mut self) -> Token<'a> {
        self.consume_number();
        let mut ch = self.chars.clone();
        if let Some((_index, '.')) = ch.next() {
            if let Some((_index, char)) = ch.next() {
                if is_digit(&Some(char)) {
                    self.advance();
                    self.consume_number();
                }
            }
        }
        self.make_token(TType::Number)
    }
}

fn check_key(word: &str, kw: &str, pos: usize, ttype: TType) -> TType {
    if word[pos..] == kw[pos..] {
        ttype
    } else {
        TType::Identifer
    }
}

fn is_identifier(char: &Option<char>) -> bool {
    match char {
        Some(c) => c.is_ascii_alphabetic() || *c == '_',
        None => false,
    }
}

fn is_digit(char: &Option<char>) -> bool {
    match char {
        Some(c) => c.is_ascii_digit(),
        None => false,
    }
}
