use crate::value::*;

#[derive(Debug)]
pub enum OpCode {
    OpConstant(u8),
    OpReturn,
}

pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>
}

impl Chunk {

    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new()
        }
    }

    pub fn write_chunk(&mut self, byte: OpCode) -> () {

        self.code.push(byte);
    }

    pub fn disassemble_chunk(&self, name: &str) {

        println!("== {} ==", name);

        let mut idx = 0;
        while idx < self.code.len() {
            idx = self.disassemble_instruction(idx);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {

        print!("{:04} ", offset);

        let instruction = &self.code[offset];

        match instruction {
            OpCode::OpConstant(index) => self.constant_instruction("OP_CONSTANT", offset, (*index).into()),
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset),
        }

    }

    pub fn simple_instruction(&self, name: &str, idx: usize) -> usize {
        println!("{}", name);
        idx + 1
    }

    pub fn add_constant(&mut self, value: Value) ->usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    fn constant_instruction(&self, name: &str, offset: usize, idx: usize) -> usize {
        print!("{} {:?} '", name, idx);
        print_value(&self.constants[idx]);
        println!("'");
        offset + 1
    }
}