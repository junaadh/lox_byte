use std::borrow::Borrow;

use crate::{error::CompileErrors, value::Value};

type OffsetWLine = (usize, usize);

#[derive(Debug, Default, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<OffsetWLine>,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        match self.lines.last() {
            Some((_, l)) if l == line.borrow() => {}
            _ => self.lines.push((self.code.len() - 1, line)),
        }
    }

    pub fn add(&mut self, val: Value) -> Result<u8, CompileErrors> {
        if self.constants.len() > (u8::MAX as usize) {
            Err(CompileErrors::TooManyConstants)
        } else {
            self.constants.push(val);
            Ok((self.constants.len() - 1) as u8)
        }
    }
}
