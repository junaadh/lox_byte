use crate::{
    chunks::Chunk,
    disassembler::Disassembler,
    error::CompileErrors,
    opcode::OpCode,
    parser::{get_rule, Parser, Precedence},
    token::TType,
    value::Value,
    vm::VM,
};

#[derive(Debug)]
pub struct Compiler<'src, 'vm> {
    pub vm: &'vm mut VM,
    pub parser: Parser<'src>,
    pub compiling_chunk: Chunk,
}

impl<'src, 'vm> Compiler<'src, 'vm> {
    pub fn new(source: &'src str, vm: &'vm mut VM) -> Self {
        Self {
            vm,
            parser: Parser::new(source),
            compiling_chunk: Chunk::default(),
        }
    }

    pub fn compile(&mut self) -> Result<(), CompileErrors> {
        self.parser.advance();

        while !self.parser.match_token(TType::Eof) {
            self.expression();
        }

        self.parser
            .consume(TType::Eof, "Expected end of expression");
        self.end_compiler();
        self.vm.chunks = self.compiling_chunk.clone();
        Ok(())
    }

    pub fn get_current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }

    pub fn end_compiler(&mut self) {
        self.emit_return();
        if cfg!(feature = "debug")
            || cfg!(debug_assertions) && self.parser.had_error && !cfg!(feature = "trace")
        {
            println!("...Dump...");
            self.compiling_chunk.disassemble("Code");
        }
    }

    pub fn emit_byte(&mut self, byte: u8) {
        let line = self.parser.previous.as_ref().unwrap().line;
        self.compiling_chunk.write(byte, line);
    }

    pub fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return.into())
    }

    pub fn emit_constant(&mut self, value: Value) {
        match self.get_current_chunk().add(value) {
            Ok(byte) => self.emit_bytes(OpCode::Constant.into(), byte),
            Err(err) => self.parser.error_at(format!("{}", err).as_str()),
        }
    }

    pub fn emit_byte_with_line(&mut self, byte: u8, line: usize) {
        self.compiling_chunk.write(byte, line);
    }

    pub fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    pub fn parse_precedence(&mut self, prec: Precedence) {
        self.parser.advance();

        let assign = prec <= Precedence::Assignment;
        match get_rule(self.parser.previous.as_ref().unwrap().ttype).prefix {
            Some(rule) => rule(self, assign),
            None => self
                .parser
                .error_at(format!("{} Expected expression.", CompileErrors::ParseError).as_str()),
        }

        while prec <= get_rule(self.parser.current.as_ref().unwrap().ttype).precedence {
            self.parser.advance();
            match get_rule(self.parser.previous.as_ref().unwrap().ttype).infix {
                Some(rule) => rule(self, assign),
                None => continue,
            }
        }

        if assign && self.parser.match_token(TType::Equal) {
            self.parser.error_at(
                format!("{} Invalid assignment target.", CompileErrors::ParseError).as_str(),
            )
        }
    }
}
