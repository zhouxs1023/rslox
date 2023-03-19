use crate::chunk::{Chunk, OpCode};
use crate::value::{print_value, Value};

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

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = 0;
        self.run()
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