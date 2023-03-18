use crate::chunk::{Chunk, OpCode};
use crate::value::print_value;

pub struct VM {
    chunk: Chunk,
    ip: usize,
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
                    return InterpretResult::Ok;
                }
                OpCode::OpConstant(index) => {
                    let constant = &self.chunk.constants[*index as usize];
                    print_value(constant);
                    print!("\n");
                }
            }
            self.ip += 1;
        }
    }

    pub fn debug_trace_execution(&self) {
        self.chunk.disassemble_instruction(self.ip);
    }
}