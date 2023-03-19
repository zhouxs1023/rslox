use crate::chunk::{Chunk, OpCode};
use crate::value::{print_value, Value};
use crate::compiler::{compile};
use std::fs;
use std::process::exit;

pub struct VM {
    pub chunk: Chunk,
    pub ip: usize,
    pub stack: Vec<Value>,
}

#[allow(dead_code)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

impl VM {

    pub fn new() -> VM {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        compile(source);
        return InterpretResult::Ok;
    }

    pub fn run(&mut self) -> InterpretResult {

        loop {
            self.debug_trace_execution();

            let opcode = &self.chunk.code[self.ip];

            match opcode {
                OpCode::OpReturn => {
                    print_value(&self.stack.pop().unwrap());
                    print!("\n");
                    return InterpretResult::Ok;
                }

                OpCode::OpAdd => self.binary_op(|a, b| a + b),

                OpCode::OpSubtract => self.binary_op(|a, b| a - b),

                OpCode::OpMultiply => self.binary_op(|a, b| a * b),

                OpCode::OpDivide => self.binary_op(|a, b| a / b),

                OpCode::OpNegate => {
                    let num = self.stack.pop().unwrap();
                    self.stack.push(-num);
                }

                OpCode::OpConstant(index) => {
                    let constant = &self.chunk.constants[*index as usize];
                    print_value(constant);
                    self.stack.push(*constant);
                    print!("\n");
                }
            }
            self.ip += 1;
        }
    }

    pub fn run_file(&mut self, path: &str) {
        let source = fs::read_to_string(path).expect("Could not open file");
        let result = self.interpret(source.as_str());

        match result {
            InterpretResult::CompileError => exit(65),
            InterpretResult::RuntimeError => exit(70),
            InterpretResult::Ok => exit(0),
        }
    }

    pub  fn binary_op(&mut self, f: fn(f64, f64) -> f64) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();

        let result = f(a, b);
        self.stack.push(result)
    }

    pub fn debug_trace_execution(&self) {

        println!("          ");
        for slot in self.stack.iter() {
            print!("[ ");
            print_value(slot);
            print!(" ]");
        }
        println!(" ");

        self.chunk.disassemble_instruction(self.ip);
    }
}