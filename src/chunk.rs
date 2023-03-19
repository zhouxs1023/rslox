use crate::value::*;

#[derive(Debug)]
pub enum OpCode {
    OpConstant(u8),
    OpNegate,
    OpReturn,
}

pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {

    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write_chunk(&mut self, byte: OpCode, line: usize) -> () {

        self.code.push(byte);
        self.lines.push(line);
    }

    #[allow(dead_code)]
    pub fn disassemble_chunk(&self, name: &str) {

        println!("== {} ==", name);

        let mut idx = 0;
        while idx < self.code.len() {
            idx = self.disassemble_instruction(idx);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {

        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("  | ");
        } else {
            print!("{} ", self.lines[offset]);
        }

        let instruction = &self.code[offset];

        match instruction {
            OpCode::OpConstant(index) => self.constant_instruction("OP_CONSTANT", offset, (*index).into()),
            OpCode::OpNegate => self.simple_instruction("OP_NEGATE", offset),
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