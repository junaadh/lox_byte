use core::fmt;
use std::{
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use crate::{
    error::{RuntimeErrors, VmErrors},
    vm::VM,
};

pub type ObjRoot<T> = Rc<HeapElement<T>>;
pub type ObjRef<T> = Weak<HeapElement<T>>;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(ObjRef<String>),
    Bool(bool),
    Nil,
}

impl Value {
    pub fn is_falsy(&self) -> bool {
        match self {
            Self::Bool(b) => !b,
            Self::Nil => true,
            _ => false,
        }
    }

    pub fn negate(&self) -> Result<Self, VmErrors> {
        let value: f64 = self.to_owned().try_into()?;
        Ok(Value::from(-value))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{num}"),
            Self::String(str) => {
                let word = &str.upgrade().unwrap().content;
                write!(f, "\"{}\"", word)
            }
            Self::Bool(bool) => write!(f, "{bool}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => Weak::ptr_eq(a, b),
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Nil, Self::Nil) => true,
            _ => false,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<ObjRef<String>> for Value {
    fn from(value: ObjRef<String>) -> Self {
        Self::String(value)
    }
}

impl TryFrom<Value> for f64 {
    type Error = VmErrors;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(n),
            _ => Err(VmErrors::RuntimeError(RuntimeErrors::TypeError(
                "number",
                value.to_string(),
            ))),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = VmErrors;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Ok(s.upgrade().unwrap().content.clone()),
            _ => Err(VmErrors::RuntimeError(RuntimeErrors::TypeError(
                "string",
                value.to_string(),
            ))),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = VmErrors;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(VmErrors::RuntimeError(RuntimeErrors::TypeError(
                "bool",
                value.to_string(),
            ))),
        }
    }
}

#[derive(Debug)]
pub struct HeapElement<T> {
    pub content: T,
}

impl<T> HeapElement<T> {
    pub fn new(content: T) -> Self {
        Self { content }
    }
}

impl<T> fmt::Display for HeapElement<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[derive(Debug)]
pub struct InternString(pub ObjRoot<String>);

// required by hashset
impl Hash for InternString {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.content.hash(h)
    }
}

impl PartialEq for InternString {
    fn eq(&self, other: &Self) -> bool {
        self.0.content == other.0.content
    }
}

// required by hashset
impl Eq for InternString {}

impl fmt::Display for InternString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.content)
    }
}

impl std::borrow::Borrow<str> for InternString {
    fn borrow(&self) -> &str {
        self.0.content.borrow()
    }
}

impl TryFrom<Value> for InternString {
    type Error = VmErrors;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(str) => Ok(Self(str.upgrade().unwrap())),
            _ => Err(VmErrors::RuntimeError(RuntimeErrors::TypeError(
                "string",
                value.to_string(),
            ))),
        }
    }
}

impl TryFrom<String> for InternString {
    type Error = VmErrors;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(Rc::new(HeapElement::<String>::new(value))))
    }
}

pub trait Objs: fmt::Display + fmt::Debug {}

impl Objs for ObjRoot<String> {}

pub fn create_string(vm: &mut VM, str: &str) -> ObjRef<String> {
    match vm.strings.get(str) {
        Some(InternString(root)) => Rc::downgrade(root),
        None => {
            let element = HeapElement::<String>::new(str.to_owned());
            let root = Rc::new(element);
            let oref = Rc::downgrade(&root);
            let intern = InternString(Rc::clone(&root));
            vm.strings.insert(intern);
            vm.objs.push(Box::new(root));
            oref
        }
    }
}
