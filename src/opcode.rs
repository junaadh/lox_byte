use core::fmt;

use crate::error::RuntimeErrors;

#[derive(Debug, Default)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    // binary
    Addition,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,

    Print,
    Jump,
    JumpIfFalse,
    Loop,

    True,
    Pop,
    GetLocal,
    SetLocal,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    Equal,
    False,
    Greater,
    Less,

    Nil,
    #[default]
    Return,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant => write!(f, "Op_Constant"),
            Self::Addition => write!(f, "Op_Addition"),
            Self::Subtract => write!(f, "Op_Subtract"),
            Self::Multiply => write!(f, "Op_Multiply"),
            Self::Divide => write!(f, "Op_Divide"),
            Self::Not => write!(f, "Op_Not"),
            Self::Negate => write!(f, "Op_Negate"),
            Self::Print => write!(f, "Op_Print"),
            Self::Jump => write!(f, "Op_Jump"),
            Self::JumpIfFalse => write!(f, "Op_JumpIfFalse"),
            Self::Loop => write!(f, "Op_Loop"),
            Self::True => write!(f, "Op_True"),
            Self::Pop => write!(f, "Op_Pop"),
            Self::GetLocal => write!(f, "Op_GetLocal"),
            Self::SetLocal => write!(f, "Op_SetLocal"),
            Self::GetGlobal => write!(f, "Op_GetGlobal"),
            Self::DefineGlobal => write!(f, "Op_DefineGlobal"),
            Self::SetGlobal => write!(f, "Op_SetGlobal"),
            Self::False => write!(f, "Op_False"),
            Self::Equal => write!(f, "Op_Equal"),
            Self::Greater => write!(f, "Op_Greater"),
            Self::Less => write!(f, "Op_Less"),
            Self::Nil => write!(f, "Op_Nil"),
            Self::Return => write!(f, "Op_Return"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = RuntimeErrors;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let last_op: u8 = Self::Return.into();
        if value < last_op + 1 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(RuntimeErrors::InvalidOpcode)
        }
    }
}
