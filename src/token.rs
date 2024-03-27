use core::fmt;

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub ttype: TType,
    pub lexeme: Option<&'a str>,
    pub line: usize,
}

impl<'a> Token<'a> {
    pub fn new(ttype: TType, lexeme: Option<&'a str>, line: usize) -> Self {
        Self {
            ttype,
            lexeme,
            line,
        }
    }
}

impl<'a> From<TType> for Token<'a> {
    fn from(value: TType) -> Self {
        Self {
            ttype: value,
            lexeme: None,
            line: 1,
        }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] Error", self.line)?;
        match self.ttype {
            TType::Eof => write!(f, " at end"),
            TType::UnexpectedCharacterError | TType::UnterminatedStringError => {
                write!(
                    f,
                    " {} at '{}'",
                    self.ttype.error_message().unwrap(),
                    self.lexeme.unwrap_or_default()
                )
            }
            _ => write!(f, " at '{}'", self.lexeme.unwrap_or_default()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TType {
    // single token
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,
    // double token
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    // literals
    Identifer,
    String,
    Number,
    // keywords
    And,
    Class,
    Else,
    False,
    True,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    // extra
    Eof,
    UnexpectedCharacterError,
    UnterminatedStringError,
}

impl TType {
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::UnexpectedCharacterError => Some("Unexpected character."),
            Self::UnterminatedStringError => Some("Unterminated string."),
            _ => None,
        }
    }
}
