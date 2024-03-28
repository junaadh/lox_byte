use crate::{
    chunks::Chunk,
    disassembler::Disassembler,
    error::CompileErrors,
    opcode::OpCode,
    parser::{get_rule, Local, Parser, Precedence},
    token::{TType, Token},
    value::{create_string, Value},
    vm::VM,
};

#[derive(Debug)]
pub struct Compiler<'src, 'vm> {
    pub vm: &'vm mut VM,
    pub parser: Parser<'src>,
    pub locals: Vec<Local<'src>>,
    pub scope_depth: usize,
    pub compiling_chunk: Chunk,
}

// macro_rules! matcher {
//     ($self: ident, $token: ident, $action: expr) => {
//         if $self.parser.match_token(TType::$token) {
//             $action
//         }
//     };
// }

impl<'src, 'vm> Compiler<'src, 'vm> {
    pub fn new(source: &'src str, vm: &'vm mut VM) -> Self {
        Self {
            vm,
            parser: Parser::new(source),
            locals: Vec::new(),
            scope_depth: 0,
            compiling_chunk: Chunk::default(),
        }
    }

    pub fn compile(&mut self) -> Result<(), CompileErrors> {
        self.parser.advance();

        while !self.parser.match_token(TType::Eof) {
            self.declaraction()
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

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.emit_byte(OpCode::Pop.into());
            self.locals.pop();
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

    pub fn block(&mut self) {
        while !self.parser.check(TType::RightBrace) && !self.parser.check(TType::Eof) {
            self.declaraction();
        }

        self.parser
            .consume(TType::RightBrace, "Unclosed '{'. Expect a '}' after block.");
    }

    pub fn var_declaration(&mut self) {
        match self.parse_variable("Expect variable name") {
            Ok(var) => {
                if self.parser.match_token(TType::Equal) {
                    self.expression();
                } else {
                    self.emit_byte(OpCode::Nil.into());
                }
                self.parser
                    .consume(TType::SemiColon, "Expect ';' after variable declaration.");
                self.define_variable(var);
            }
            Err(err) => self.parser.error_at(format!("{err}").as_str()),
        }
    }

    pub fn expression_statement(&mut self) {
        self.expression();
        self.parser
            .consume(TType::SemiColon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    pub fn print_statement(&mut self) {
        self.expression();
        self.parser
            .consume(TType::SemiColon, "Expect ';' after print statement.");
        self.emit_byte(OpCode::Print.into());
    }

    pub fn synchronize(&mut self) {
        self.parser.set_panic(true);

        while self.parser.current.as_ref().unwrap().ttype != TType::Eof {
            if self.parser.previous.as_ref().unwrap().ttype == TType::SemiColon {
                return;
            }
            match self.parser.current.as_ref().unwrap().ttype {
                TType::Class
                | TType::Fun
                | TType::Var
                | TType::For
                | TType::If
                | TType::While
                | TType::Print
                | TType::Return => return,
                _ => {}
            }
            self.parser.advance();
        }
    }

    pub fn declaraction(&mut self) {
        // matcher!(self, Var, self.var_declaration());
        if self.parser.match_token(TType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.get_panic() {
            self.synchronize();
        }
    }

    pub fn statement(&mut self) {
        // matcher!(self, Print, self.print_statement());
        if self.parser.match_token(TType::Print) {
            self.print_statement();
        } else if self.parser.match_token(TType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
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
            // match get_rule(self.parser.previous.as_ref().unwrap().ttype).infix {
            //     Some(rule) => rule(self, assign),
            //     None => continue,
            // }
            get_rule(self.parser.previous.as_ref().unwrap().ttype)
                .infix
                .unwrap()(self, assign)
        }

        if assign && self.parser.match_token(TType::Equal) {
            self.parser.error_at(
                format!("{} Invalid assignment target.", CompileErrors::ParseError).as_str(),
            )
        }
    }

    pub fn identififer_constant(&mut self, t: Option<Token<'src>>) -> Result<u8, CompileErrors> {
        let name = &t.unwrap().lexeme.unwrap();
        let str = create_string(self.vm, name);
        self.get_current_chunk().add(str.into())
    }

    fn resolve_local(&mut self, name: &'src str) -> Option<u8> {
        for (index, local) in self.locals.iter().enumerate() {
            if local.name == name {
                return Some(index as u8);
            }
        }
        None
    }

    fn add_local(&mut self, name: &'src str) {
        if self.locals.len() == u8::MAX as usize + 1 {
            self.parser
                .error_at(format!("{}", CompileErrors::TooManyLocals).as_str());
            return;
        }

        let local = Local::new(name, self.scope_depth);
        self.locals.push(local);
    }

    pub fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        let name = self.parser.previous.as_ref().unwrap().lexeme.unwrap();

        for local in self.locals.iter().rev() {
            if local.depth < self.scope_depth {
                break;
            }
            if local.name == name {
                self.parser
                    .error_at(format!("{}", CompileErrors::DuplicateName).as_str());
                return;
            }
        }

        self.add_local(name);
    }

    pub fn parse_variable(&mut self, error: &str) -> Result<u8, CompileErrors> {
        self.parser.consume(TType::Identifer, error);
        self.declare_variable();
        if self.scope_depth > 0 {
            return Ok(0_u8);
        }
        self.identififer_constant(self.parser.previous.clone())
    }

    fn mark_initialized(&mut self) {
        let last = self.locals.len() - 1;
        self.locals[last].depth = self.scope_depth;
    }

    pub fn define_variable(&mut self, global: u8) {
        if self.scope_depth > 0 {
            self.mark_initialized();
        } else {
            self.emit_bytes(OpCode::DefineGlobal.into(), global);
        }
    }

    pub fn named_variable(&mut self, token: Option<Token<'src>>, can_assign: bool) {
        let name = token.as_ref().unwrap().lexeme.unwrap();
        let (get_op, set_op, arg) = match self.resolve_local(name) {
            Some(index) => (OpCode::GetLocal, OpCode::SetLocal, index),
            None => (
                OpCode::GetGlobal,
                OpCode::SetGlobal,
                self.identififer_constant(token)
                    .map_err(|err| self.parser.error_at(format!("{}", err).as_str()))
                    .unwrap(),
            ),
        };

        if can_assign && self.parser.match_token(TType::Equal) {
            self.expression();
            self.emit_bytes(set_op.into(), arg);
        } else {
            self.emit_bytes(get_op.into(), arg);
        }
    }
}
