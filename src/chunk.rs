
#[derive(Debug)]
pub enum OpCode {
    OpReturn,
}

pub struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {

    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
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
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset),
        }

    }

    pub fn simple_instruction(&self, name: &str, idx: usize) -> usize {
        println!("{}", name);
        idx + 1
    }
}