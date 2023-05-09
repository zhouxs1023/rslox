use std::{env, io};
use std::io::{stdout, Write};
use crate::vm::{VM, InterpretResult};
use std::process::exit;
use std::fs;

mod chunk;
mod value;
mod vm;
mod compiler;
mod scanner;
mod function;

fn main() {

    let mut argv = env::args();
    let mut vm = VM::new();

    match argv.len() {
        1 => repl(&mut vm),
        2 => run_file(&argv.nth(1).expect("Could not parse argv")),
        _ => {
            eprintln!("Usage: rslox [path]");
            exit(64);
        }
    }
}

pub fn repl(vm: &mut VM) {
    let mut buf = String::new();
    let stdin = io::stdin();
    loop {
        print!("> ");
        stdout().flush().unwrap();

        match stdin.read_line(&mut buf) {
            Ok(0) | Err(_) => {
                println!();
                break;
            }
            Ok(_) => {
                vm.interpret(&buf);
            }
        }
        buf.clear()
    }
}

fn run_file(path: &str) {
    let mut vm = VM::new();
    let source = fs::read_to_string(path).expect("Could not open file");
    let result = vm.interpret(source.as_str());

    match result {
        InterpretResult::CompileError => exit(65),
        InterpretResult::RuntimeError => exit(70),
        InterpretResult::Ok => exit(0),
    }
}