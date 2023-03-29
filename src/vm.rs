use crate::chunk::{Chunk, OpCode};
use crate::value::{print_value, Value};
use crate::compiler::Parser;

pub struct VM {
    pub chunk: Chunk,
    pub ip: usize,
    pub stack: Vec<Value>,
}

#[derive(PartialEq, Debug)]
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
        let mut parser = Parser::new(source);

        if !parser.compile() {
            return InterpretResult::CompileError;
        }

        self.chunk = parser.chunk;
        self.ip = 0; // or self.chunk.code?

        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {

        loop {
           self.debug_trace_execution();

            let opcode = &self.chunk.code[self.ip];

            match opcode {
                OpCode::OpReturn => {
                    print_value(&self.stack.pop().unwrap());
                    println!();
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