use crate::{chunks::Chunk, opcode::OpCode, value::Value};

pub trait Disassembler {
    fn disassemble(&self, name: &str);
}

impl Disassembler for Chunk {
    #[allow(unused_variables)]
    fn disassemble(&self, name: &str) {
        if cfg!(feature = "debug") || cfg!(debug_assertions) {
            println!("=={}==", name);
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

    pub fn read_constant(&mut self) -> Value {
        let off = self.read();
        self.chunk.constants[off as usize].clone()
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
        print!("{:04} ", self.offset);
        if self.offset > 0 && self.get_line() == self.get_prev_line() {
            print!("   | ");
        } else {
            print! {"{:04} ", self.get_line().unwrap()};
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
                OpCode::True => self.simple_instruction(&op),
                OpCode::Pop => self.simple_instruction(&op),
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
            Err(err) => println!("{}", err),
        }
    }

    fn simple_instruction(&self, instruction: &OpCode) {
        println!("{}", instruction)
    }

    fn constant_instruction(&mut self, instruction: &OpCode) {
        let constant = self.read();

        println!(
            "{:<16} {:<4} {}",
            instruction, constant, self.chunk.constants[constant as usize]
        );
    }
}
