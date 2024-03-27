#![allow(unused_variables)]

use crate::{
    compiler::Compiler,
    error::CompileErrors,
    opcode::OpCode,
    scanner::Scanner,
    token::{TType, Token},
    value::create_string,
};

#[derive(Debug)]
pub struct Parser<'src> {
    pub scanner: Scanner<'src>,
    pub current: Option<Token<'src>>,
    pub previous: Option<Token<'src>>,

    pub had_error: bool,
    panic_mode: bool,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            scanner: Scanner::new(source),
            previous: None,
            current: None,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn check(&mut self, tt: TType) -> bool {
        if let Some(t) = &self.current {
            t.ttype == tt
        } else {
            false
        }
    }

    pub fn match_token(&mut self, tt: TType) -> bool {
        if !self.check(tt) {
            return false;
        }
        self.advance();
        true
    }

    pub fn advance(&mut self) {
        self.previous = self.current.take();
        // println!("{:?}", self.previous);
        loop {
            let token = self.scanner.scan_token();
            let error = TType::error_message(&token.ttype);
            self.current = Some(token.clone());
            match error {
                Some(e) => self.error_at(e),
                None => break,
            }
        }
    }

    pub fn error_at(&mut self, msg: &str) {
        if self.panic_mode {
            return;
        }
        self.had_error = true;
        self.panic_mode = true;
        if let Some(tok) = &self.current {
            println!("{}: {}", tok, msg);
        }
    }

    pub fn consume(&mut self, tt: TType, msg: &str) {
        if let Some(t) = &self.current {
            if t.ttype == tt {
                self.advance();
                return;
            }
        }
        self.error_at(msg);
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub enum Precedence {
    #[default]
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

type ParserFn = fn(&mut Compiler<'_, '_>, bool);

#[derive(Debug, Default)]
pub struct ParseRule {
    pub prefix: Option<ParserFn>,
    pub infix: Option<ParserFn>,
    pub precedence: Precedence,
}

pub fn get_rule(tt: TType) -> ParseRule {
    match tt {
        TType::LeftParen => ParseRule {
            prefix: Some(grouping),
            // infix: Some(call),
            // precedence: Precedence::Call,
            ..ParseRule::default()
        },
        TType::Minus => ParseRule {
            prefix: Some(unary),
            infix: Some(binary),
            precedence: Precedence::Term,
        },
        TType::Plus => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Term,
        },
        TType::Slash => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Factor,
        },
        TType::Star => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Factor,
        },
        TType::Bang => ParseRule {
            prefix: Some(unary),
            ..ParseRule::default()
        },
        TType::BangEqual => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Equality,
        },
        TType::EqualEqual => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Equality,
        },
        TType::Greater => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Comparison,
        },
        TType::GreaterEqual => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Comparison,
        },
        TType::Less => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Comparison,
        },
        TType::LessEqual => ParseRule {
            prefix: None,
            infix: Some(binary),
            precedence: Precedence::Comparison,
        },
        TType::Identifer => ParseRule {
            prefix: Some(variable),
            ..ParseRule::default()
        },
        TType::String => ParseRule {
            prefix: Some(string),
            ..ParseRule::default()
        },
        TType::Number => ParseRule {
            prefix: Some(number),
            ..ParseRule::default()
        },
        TType::False => ParseRule {
            prefix: Some(literal),
            ..ParseRule::default()
        },
        TType::Nil => ParseRule {
            prefix: Some(literal),
            ..ParseRule::default()
        },
        TType::True => ParseRule {
            prefix: Some(literal),
            ..ParseRule::default()
        },
        TType::And => ParseRule {
            prefix: None,
            infix: Some(and_op),
            precedence: Precedence::And,
        },
        TType::Or => ParseRule {
            prefix: None,
            infix: Some(or_op),
            precedence: Precedence::Or,
        },
        _ => ParseRule::default(),
    }
}

fn grouping(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    cc.expression();
    cc.parser
        .consume(TType::RightParen, "Unclosed ')' parenthesis.");
}

fn unary(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    let token = cc.parser.previous.as_ref().unwrap();
    let op = token.ttype;
    let line = token.line;
    cc.parse_precedence(Precedence::Unary);
    match op {
        TType::Bang => cc.emit_byte_with_line(OpCode::Not.into(), line),
        TType::Minus => cc.emit_byte_with_line(OpCode::Negate.into(), line),
        _ => unreachable!(),
    }
}

fn binary(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    let tt = cc.parser.previous.as_ref().unwrap().ttype;
    let precedence: usize = get_rule(tt).precedence.into();
    cc.parse_precedence(Precedence::try_from(precedence + 1).unwrap());

    match tt {
        TType::Plus => cc.emit_byte(OpCode::Addition.into()),
        TType::Minus => cc.emit_byte(OpCode::Subtract.into()),
        TType::Star => cc.emit_byte(OpCode::Multiply.into()),
        TType::Slash => cc.emit_byte(OpCode::Divide.into()),
        TType::BangEqual => cc.emit_bytes(OpCode::Equal.into(), OpCode::Not.into()),
        TType::EqualEqual => cc.emit_byte(OpCode::Equal.into()),
        TType::Greater => cc.emit_byte(OpCode::Greater.into()),
        TType::GreaterEqual => cc.emit_bytes(OpCode::Less.into(), OpCode::Not.into()),
        TType::Less => cc.emit_byte(OpCode::Less.into()),
        TType::LessEqual => cc.emit_bytes(OpCode::Greater.into(), OpCode::Not.into()),
        _ => unreachable!(),
    }
}

#[allow(dead_code)]
fn call(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    unimplemented!("call")
}

fn number(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    let number: f64 = cc
        .parser
        .previous
        .as_ref()
        .unwrap()
        .lexeme
        .unwrap()
        .parse::<f64>()
        .unwrap();
    cc.emit_constant(number.into())
}

fn string(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    let vm = &mut cc.vm;
    let prev = cc.parser.previous.as_ref().unwrap().clone().lexeme.unwrap();
    let w = create_string(vm, &prev[1..prev.len() - 1]);
    cc.emit_constant(w.into())
}

fn variable(cc: &mut Compiler<'_, '_>, can_assign: bool) {
    unimplemented!("variable")
}

fn literal(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    match cc.parser.previous.as_ref().unwrap().ttype {
        TType::False => cc.emit_byte(OpCode::False.into()),
        TType::True => cc.emit_byte(OpCode::True.into()),
        TType::Nil => cc.emit_byte(OpCode::Nil.into()),
        _ => unreachable!(),
    }
}

fn and_op(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    unimplemented!("and")
}
fn or_op(cc: &mut Compiler<'_, '_>, _can_assign: bool) {
    unimplemented!("or")
}

impl From<Precedence> for usize {
    fn from(value: Precedence) -> Self {
        value as usize
    }
}

impl TryFrom<usize> for Precedence {
    type Error = CompileErrors;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let last_prec: usize = Precedence::Primary.into();
        if value < last_prec + 1 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(CompileErrors::InvalidPrecedence)
        }
    }
}
