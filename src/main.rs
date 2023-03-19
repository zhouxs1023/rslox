use std::{env, io};
use std::io::{stdout, Write};
use crate::vm::VM;
use std::process::exit;

mod chunk;
mod value;
mod vm;
mod compiler;

fn main() {

    let mut argv = env::args();
    let mut vm = VM::new();

    match argv.len() {
        1 => repl(&mut vm),
        2 => vm.run_file(&argv.nth(1).expect("Could not parse argv")),
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
            Ok(_) => vm.interpret(&buf),
        };
    }
    buf.clear();
}
