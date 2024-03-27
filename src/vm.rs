use std::collections::HashSet;

use crate::{
    chunks::Chunk,
    compiler::Compiler,
    disassembler::TracingIp,
    error::{RuntimeErrors, VmErrors},
    opcode::OpCode,
    value::{InternString, Objs, Value},
};

type InterpretRes = Result<(), VmErrors>;
type VMRes<T> = Result<T, VmErrors>;

#[derive(Debug)]
pub struct VM {
    pub stack: Vec<Value>,
    pub objs: Vec<Box<dyn Objs>>,
    // interned string db
    pub strings: HashSet<InternString>,
    pub chunks: Chunk,
}

impl VM {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            stack: Vec::<Value>::new(),
            objs: Vec::new(),
            strings: HashSet::<InternString>::new(),
            chunks: Chunk::default(),
        }
    }

    pub fn interpret(&mut self, src: &str) -> InterpretRes {
        let mut cc = Compiler::new(src, self);
        cc.compile().map_err(VmErrors::CompileError)?;
        let result = self.run();
        if let Err(VmErrors::RuntimeError(e)) = result {
            eprintln!("Runtime Error: {}", e);
        }
        Ok(())
    }

    pub fn run(&mut self) -> InterpretRes {
        macro_rules! binary_op {
            ($op: tt) => {{
                let b: f64 = self.pop()?.try_into()?;
                let a: f64 = self.pop()?.try_into()?;
                self.stack.push((a $op b).into());
            }};
        }

        if cfg!(feature = "trace") {
            println!("Execution Trace");
        }
        let chunk = self.chunks.clone();
        let mut ip = TracingIp::new(&chunk, 0);
        while ip.valid() {
            if cfg!(feature = "trace") {
                println!("{:?}\n", self.stack);
                ip.clone().disassemble_instruction();
            }
            let byte = ip.read();
            match OpCode::try_from(byte) {
                Ok(op) => match op {
                    OpCode::Constant => {
                        let val = ip.chunk.constants[ip.read() as usize].clone();
                        self.stack.push(val);
                    }
                    OpCode::Addition => {
                        let val2 = self.pop()?;
                        let val1 = self.pop()?;
                        match (&val1, &val2) {
                            (Value::String(v1), Value::String(v2)) => {
                                let v1 = &v1.upgrade().unwrap().content;
                                let v2 = &v2.upgrade().unwrap().content;
                                let concat = format!("{}{}", v1, v2);
                                self.stack.push(concat.into());
                            }
                            (Value::String(v1), Value::Number(v2)) => {
                                let v1 = &v1.upgrade().unwrap().content;
                                let concat = format!("{}{}", v1, v2);
                                self.stack.push(concat.into());
                            }
                            (Value::Number(v1), Value::String(v2)) => {
                                let v2 = &v2.upgrade().unwrap().content;
                                let concat = format!("{}{}", v1, v2);
                                self.stack.push(concat.into());
                            }
                            (Value::Number(v1), Value::Number(v2)) => {
                                let concat = v1 + v2;
                                self.stack.push(concat.into());
                            }
                            _ => {
                                return Err(VmErrors::RuntimeError(RuntimeErrors::InvalidAddition(
                                    val1.to_string(),
                                    val2.to_string(),
                                )))
                            }
                        }
                    }
                    OpCode::Subtract => binary_op!(-),
                    OpCode::Multiply => binary_op!(*),
                    OpCode::Divide => binary_op!(/),
                    OpCode::Not => {
                        let bool = self.pop()?.is_falsy();
                        self.stack.push(bool.into())
                    }
                    OpCode::Negate => {
                        let val = self.pop()?;
                        self.stack.push(val.negate()?)
                    }
                    OpCode::True => self.stack.push(true.into()),
                    OpCode::False => self.stack.push(false.into()),
                    OpCode::Equal => {
                        let a = self.pop()?;
                        let b = self.pop()?;
                        self.stack.push((a == b).into())
                    }
                    OpCode::Greater => binary_op!(>),
                    OpCode::Less => binary_op!(<),
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::Return => {
                        let val = self.pop()?;
                        println!("{}", val);
                        return Ok(());
                    }
                },
                Err(err) => return Err(VmErrors::RuntimeError(err)),
            }
        }
        Ok(())
        // match chunk_slice.next() {
        //     Some(x) => {}
        //     None => Err(VmErrors::RuntimeError(RuntimeErrors::InvalidOpcode)),
        // }
    }

    fn pop(&mut self) -> VMRes<Value> {
        match self.stack.pop() {
            Some(x) => Ok(x),
            None => Err(VmErrors::RuntimeError(RuntimeErrors::StackUnderFlow)),
        }
    }

    #[allow(dead_code)]
    fn peek(&mut self, distance: usize) -> Value {
        self.stack[self.stack.len() - 1 - distance].clone()
    }

    // pub fn interpret(&mut self, src: &str) -> InterpretRes {}
}
