use core::fmt;

#[derive(Debug, Clone)]
pub enum CompileErrors {
    TooManyConstants,
    CantNegateNoneNumbers,
    ParseError,
    InvalidPrecedence,
    TooManyLocals,
    DuplicateName,
    UninitializedLocal,
}

impl fmt::Display for CompileErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyConstants => {
                write!(f, "Too many constants in one chunk. Chunk overloaded.")
            }
            Self::CantNegateNoneNumbers => write!(f, "Cannot use unary operator on none numbers"),
            Self::ParseError => write!(f, "Parse Error."),
            Self::InvalidPrecedence => write!(f, "Cannot convert usize to Precedence"),
            Self::TooManyLocals => write!(f, "Too many local variables in function"),
            Self::DuplicateName => write!(f, "Already a variable in scope with this name."),
            Self::UninitializedLocal => write!(f, "Local hasn't been initialized yet."),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeErrors {
    InvalidOpcode,
    StackUnderFlow,
    TypeError(&'static str, String),
    InvalidAddition(String, String),
    UndefinedVariable(String),
}

impl fmt::Display for RuntimeErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpcode => write!(f, "Cannot convert byte to Opcode."),
            Self::StackUnderFlow => write!(f, "Attempted to pop an empty stack."),
            Self::TypeError(t, v) => write!(f, "Expected a {}, but found value {}", t, v),
            Self::InvalidAddition(v1, v2) => write!(f, "Cannot add {} and {}", v1, v2),
            Self::UndefinedVariable(v) => write!(f, "Value {}, is not defined.", v),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VmErrors {
    CompileError(CompileErrors),
    RuntimeError(RuntimeErrors),
}
