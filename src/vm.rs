use std::collections::{HashMap, HashSet};

use crate::{
    chunks::Chunk,
    compiler::Compiler,
    disassembler::TracingIp,
    error::{RuntimeErrors, VmErrors},
    opcode::OpCode,
    value::{create_string, InternString, Objs, Value},
};

type InterpretRes = Result<(), VmErrors>;
type VMRes<T> = Result<T, VmErrors>;

#[derive(Debug)]
pub struct VM {
    pub stack: Vec<Value>,
    pub objs: Vec<Box<dyn Objs>>,
    // interned string db
    pub strings: HashSet<InternString>,
    pub globals: HashMap<InternString, Value>,
    pub chunks: Chunk,
}

impl VM {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            stack: Vec::<Value>::new(),
            objs: Vec::new(),
            strings: HashSet::<InternString>::new(),
            globals: HashMap::new(),
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

        macro_rules! string {
            ($a: expr, $b: expr) => {
                create_string(self, format!("{}{}", $a, $b).as_str())
            };
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
                                let str = string!(v1, v2);
                                self.stack.push(str.into());
                            }
                            (Value::String(v1), Value::Number(v2)) => {
                                let v1 = &v1.upgrade().unwrap().content;
                                let str = string!(v1, v2);
                                self.stack.push(str.into());
                            }
                            (Value::Number(v1), Value::String(v2)) => {
                                let v2 = &v2.upgrade().unwrap().content;
                                let str = string!(v1, v2);
                                self.stack.push(str.into());
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
                    OpCode::Print => println!("{}", self.pop()?),
                    OpCode::True => self.stack.push(true.into()),
                    OpCode::Pop => {
                        self.pop()?;
                    }
                    OpCode::GetGlobal => {
                        let val = ip.read_constant();
                        let str: InternString = val.clone().try_into()?;
                        match self.globals.get(&str) {
                            Some(str) => self.stack.push(str.clone()),
                            None => {
                                return Err(VmErrors::RuntimeError(
                                    RuntimeErrors::UndefinedVariable(val.to_string()),
                                ))
                            }
                        }
                    }
                    OpCode::DefineGlobal => {
                        let val = ip.read_constant();
                        let str: InternString = val.try_into()?;
                        self.globals.insert(str, self.peek(0));
                        self.pop()?;
                    }
                    OpCode::SetGlobal => {
                        let val = ip.read_constant();
                        let str: InternString = val.clone().try_into()?;
                        /*cause double borrow*/
                        // let peek = self.peek(0);
                        // if let Entry::Occupied(mut e) = self.globals.entry(str) {
                        //     e.insert(peek);
                        #[allow(clippy::map_entry)]
                        if self.globals.contains_key(&str) {
                            self.globals.insert(str, self.peek(0));
                        } else {
                            return Err(VmErrors::RuntimeError(RuntimeErrors::UndefinedVariable(
                                val.to_string(),
                            )));
                        }
                    }
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

    fn peek(&self, distance: usize) -> Value {
        // println!("{:#?}", self.stack);
        // println!("{}", self.stack.len());
        self.stack[self.stack.len() - 1 - distance].clone()
    }

    // pub fn interpret(&mut self, src: &str) -> InterpretRes {}
}
