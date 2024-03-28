use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process,
};

use vm::VM;

use crate::error::VmErrors;

pub mod chunks;
pub mod compiler;
pub mod disassembler;
pub mod error;
pub mod macros;
pub mod memory;
pub mod opcode;
pub mod parser;
pub mod scanner;
pub mod token;
pub mod value;
pub mod vm;

fn main() {
    let mut vm = VM::new();

    let mut args = env::args();
    match args.len() {
        0..=1 => repl(&mut vm),
        2 => {
            let file = args.nth(1).unwrap_or_default();
            run_file(file, &mut vm);
        }
        _ => cprintln!(LightRed, "Usage: lox_byte [file_name]"),
    }
}

fn repl(vm: &mut VM) {
    let mut buffer = String::new();
    cprintln!(LightPurple, "{}", "Welcome to lox_byte repl.");
    cprintln!(Purple, "\tExit -> ctrl + d");
    loop {
        cprint!(LightCyan, "{}", "lox");
        cprint!(LightBlue, " > ");
        io::stdout().flush().expect("Failed to flush stdout");
        buffer.clear();
        match io::stdin().read_line(&mut buffer) {
            Ok(0) => {
                println!();
                cprintln!(Cyan, "Exiting... Goodbye...");
                process::exit(0);
            }
            Ok(_) => vm.interpret(&buffer).unwrap_or(()),
            Err(err) => {
                eprintln!("failed to get input: {}", err);
                continue;
            }
        }
    }
}

fn run_file(path: String, vm: &mut VM) {
    let mut file = File::open(path).expect("Failed to open file");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .expect("Failed to read file");
    match vm.interpret(&buffer) {
        Ok(()) => process::exit(0),
        Err(VmErrors::CompileError(e)) => {
            println!("Compile Error: {}", e);
            process::exit(69)
        }
        Err(VmErrors::RuntimeError(e)) => {
            println!("Compile Error: {}", e);
            process::exit(69)
        }
    }
}
