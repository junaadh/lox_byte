use crate::{chunks::Chunk, cprint, cprintln, opcode::OpCode, value::Value};

pub trait Disassembler {
    fn disassemble(&self, name: &str);
}

impl Disassembler for Chunk {
    #[allow(unused_variables)]
    fn disassemble(&self, name: &str) {
        if cfg!(feature = "debug") || cfg!(debug_assertions) {
            cprintln!(Red, "=={}==", name);
            let mut ip = TracingIp::new(self, 0);
            while ip.valid() {
                ip.disassemble_instruction();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TracingIp<'a> {
    pub chunk: &'a Chunk,
    pub offset: usize,
    pub line: Option<usize>,
}

impl<'a> TracingIp<'a> {
    pub fn new(chunk: &'a Chunk, offset: usize) -> Self {
        Self {
            chunk,
            offset,
            line: None,
        }
    }

    pub fn valid(&self) -> bool {
        self.offset < self.chunk.code.len()
    }

    pub fn read(&mut self) -> u8 {
        let result = self.chunk.code[self.offset];
        self.line = self.get_line();
        self.offset += 1;
        result
    }

    pub fn read_short(&mut self) -> u16 {
        let high = self.read() as u16;
        let low = self.read() as u16;
        (high >> 8) | low
    }

    pub fn read_constant(&mut self) -> Value {
        let off = self.read();
        self.chunk.constants[off as usize].clone()
    }

    pub fn peek(&self, distance: usize) -> Value {
        // println!("{:#?}", self.stack);
        // println!("{}", self.stack.len());
        self.chunk.constants[self.chunk.constants.len() - 1 - distance].clone()
    }

    pub fn get_line(&self) -> Option<usize> {
        let mut line = None;

        for &(off, l) in self.chunk.lines.iter() {
            if off > self.offset {
                break;
            }
            line = Some(l);
        }
        line
    }

    fn get_prev_line(&self) -> Option<usize> {
        let mut line = None;

        for &(off, l) in self.chunk.lines.iter() {
            let offft = self.offset - if self.offset == 0 { 0 } else { 1 };
            if off > offft {
                break;
            }
            line = Some(l);
        }
        line
    }

    pub fn disassemble_instruction(&mut self) {
        cprint!(Green, "{:04} ", self.offset);
        if self.offset > 0 && self.get_line() == self.get_prev_line() {
            cprint!(LightPurple, "   | ");
        } else {
            cprint! {LightPurple,"{:04} ", self.get_line().unwrap()};
        }
        let byte = self.read();
        match OpCode::try_from(byte) {
            Ok(op) => match op {
                OpCode::Constant => self.constant_instruction(&op),
                OpCode::Addition => self.simple_instruction(&op),
                OpCode::Subtract => self.simple_instruction(&op),
                OpCode::Multiply => self.simple_instruction(&op),
                OpCode::Divide => self.simple_instruction(&op),
                OpCode::Not => self.simple_instruction(&op),
                OpCode::Negate => self.simple_instruction(&op),
                OpCode::Print => self.simple_instruction(&op),
                OpCode::Jump => self.jump_instruction(&op, 1),
                OpCode::JumpIfFalse => self.jump_instruction(&op, 1),
                OpCode::Loop => self.jump_instruction(&op, -1),
                OpCode::True => self.simple_instruction(&op),
                OpCode::Pop => self.simple_instruction(&op),
                OpCode::GetLocal => self.byte_instruction(&op),
                OpCode::SetLocal => self.byte_instruction(&op),
                OpCode::GetGlobal => self.constant_instruction(&op),
                OpCode::DefineGlobal => self.constant_instruction(&op),
                OpCode::SetGlobal => self.constant_instruction(&op),
                OpCode::False => self.simple_instruction(&op),
                OpCode::Equal => self.simple_instruction(&op),
                OpCode::Greater => self.simple_instruction(&op),
                OpCode::Less => self.simple_instruction(&op),
                OpCode::Nil => self.simple_instruction(&op),
                OpCode::Return => self.simple_instruction(&op),
            },
            Err(err) => cprintln!(LightRed, "{}", err),
        }
    }

    fn simple_instruction(&self, instruction: &OpCode) {
        cprintln!(Cyan, "{}", instruction)
    }

    fn constant_instruction(&mut self, instruction: &OpCode) {
        let constant = self.read();

        cprintln!(
            Cyan,
            "{:<16} {:<4} {}",
            instruction,
            constant,
            self.chunk.constants[constant as usize]
        );
    }

    fn byte_instruction(&mut self, instruction: &OpCode) {
        let slot = self.read();
        cprintln!(Cyan, "{:<16} {:<4}", instruction, slot);
    }

    fn jump_instruction(&mut self, instruction: &OpCode, sign: isize) {
        let jump = self.read_short() as isize;
        cprintln!(
            Cyan,
            "{:<16} {:4} -> {:4}",
            instruction,
            jump,
            self.offset as isize + jump * sign
        );
    }
}
