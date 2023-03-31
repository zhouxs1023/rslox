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
                    print_value(&self.stack.pop().expect("Empty stack"));
                    println!();
                    return InterpretResult::Ok;
                }

                OpCode::OpConstant(index) => {
                    let constant = &self.chunk.constants[*index as usize];
                    print_value(constant);
                    self.stack.push(*constant);
                    print!("\n");
                }

                OpCode::OpNegate => match self.stack.get(self.stack.len() - 1).expect("Failed to peek") {
                    Value::Number(val) => {
                        let neg_val = -*val;
                        self.stack.pop();
                        self.stack.push(Value::Number(neg_val));
                    },
                    _ => {
                        return self.runtime_error("Operand must be a number.");
                    }
                },

                OpCode::OpAdd => self.binary_op(|a, b| a + b),
                OpCode::OpSubtract => self.binary_op(|a, b| a - b),
                OpCode::OpMultiply => self.binary_op(|a, b| a * b),
                OpCode::OpDivide => self.binary_op(|a, b| a / b),
            }
            self.ip += 1;
        }
    }

    pub  fn binary_op(&mut self, f: fn(f64, f64) -> f64) {
        let b = self.stack.pop().expect("Empty stack");
        let a = self.stack.pop().expect("Empty stack");

        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let result = f(a, b);
                self.stack.push(Value::Number(result));
            }
            _ => {
                self.runtime_error("Operands must be two numbers or two strings");
            }
        }
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

    fn runtime_error(&self, msg: &str) -> InterpretResult {
        eprintln!("{}", msg);

        let instruction = self.ip - 1;
        let line = self.chunk.lines[instruction];
        eprintln!("[line {}] in script", line);
        InterpretResult::RuntimeError
    }
}