use crate::value::*;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    OpConstant,
    OpNil,
    OpTrue,
    OpFalse,
    OpPop,
    OpDefineGlobal,
    OpGetLocal,
    OpSetLocal,
    OpGetGlobal,
    OpSetGlobal,
    OpEqual,
    OpGreater,
    OpLess,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNot,
    OpNegate,
    OpPrint,
    OpJumpIfFalse,
    OpJump,
    OpReturn,
}

impl From<u8> for OpCode {
    fn from(code: u8) -> Self {
        match code {
            0 => OpCode::OpConstant,
            1 => OpCode::OpNil,
            2 => OpCode::OpTrue,
            3 => OpCode::OpFalse,
            4 => OpCode::OpPop,
            5 => OpCode::OpDefineGlobal,
            6 => OpCode::OpGetLocal,
            7 => OpCode::OpSetLocal,
            8 => OpCode::OpGetGlobal,
            9 => OpCode::OpSetGlobal,
            10 => OpCode::OpEqual,
            11 => OpCode::OpGreater,
            12 => OpCode::OpLess,
            13 => OpCode::OpAdd,
            14 => OpCode::OpSubtract,
            15 => OpCode::OpMultiply,
            16 => OpCode::OpDivide,
            17 => OpCode::OpNot,
            18 => OpCode::OpNegate,
            19 => OpCode::OpPrint,
            20 => OpCode::OpJumpIfFalse,
            21 => OpCode::OpJump,
            22 => OpCode::OpReturn,
            _ => unimplemented!("Invalid opcode {}", code),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        code as u8
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
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

    pub fn write_byte(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte.into());
        self.lines.push(line);
    }

    pub fn write_u8(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn read_byte(&self, offset: usize) -> u8 {
        self.code[offset].into()
    }

    pub fn add_constant(&mut self, val: Value) -> Option<u8> {
        self.constants.push(val);
        u8::try_from(self.constants.len() - 1).ok()
    }

    pub fn get_constant(&self, idx: usize) -> &Value {
        self.constants.get(idx).expect("None Value!!!")
    }

    #[allow(dead_code)]
    pub fn disassemble_chunk<T: ToString>(&self, name: T) {
        println!("== {} ==", name.to_string());

        let mut idx = 0;
        while idx < self.code.len() {
            idx = self.disassemble_instruction(idx);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {

        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("{:>4} ", "|"); // right justify
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction = &self.code[offset].into();

        match instruction {
            OpCode::OpConstant => self.constant_instruction("OP_CONSTANT", offset),
            OpCode::OpNil => self.simple_instruction("OP_NIL", offset),
            OpCode::OpTrue => self.simple_instruction("OP_TRUE", offset),
            OpCode::OpFalse => self.simple_instruction("OP_FALSE", offset),
            OpCode::OpPop => self.simple_instruction("OP_POP", offset),
            OpCode::OpDefineGlobal => self.constant_instruction("OP_DEFINE_GLOBAL", offset),
            OpCode::OpGetLocal => self.byte_instruction("OP_GET_LOCAL", offset),
            OpCode::OpSetLocal => self.byte_instruction("OP_SET_LOCAL", offset),
            OpCode::OpGetGlobal => self.constant_instruction("OP_GET_GLOBAL", offset),
            OpCode::OpSetGlobal => self.constant_instruction("OP_SET_GLOBAL", offset),
            OpCode::OpEqual => self.simple_instruction("OP_EQUAL", offset),
            OpCode::OpGreater => self.simple_instruction("OP_GREATER", offset),
            OpCode::OpLess => self.simple_instruction("OP_LESS", offset),
            OpCode::OpAdd => self.simple_instruction("OP_ADD", offset),
            OpCode::OpSubtract => self.simple_instruction("OP_SUBTRACT", offset),
            OpCode::OpMultiply => self.simple_instruction("OP_MULTIPLY", offset),
            OpCode::OpDivide => self.simple_instruction("OP_DIVIDE", offset),
            OpCode::OpNot => self.simple_instruction("OP_NOT", offset),
            OpCode::OpNegate => self.simple_instruction("OP_NEGATE", offset),
            OpCode::OpPrint => self.simple_instruction("OP_PRINT", offset),
            OpCode::OpJumpIfFalse => self.jump_instruction("OP_JUMP_IF_FALSE", 1, offset),
            OpCode::OpJump => self.jump_instruction("OP_JUMP", 1, offset),
            OpCode::OpReturn => self.simple_instruction("OP_RETURN", offset),
        }

    }

    pub fn simple_instruction(&self, name: &str, idx: usize) -> usize {
        println!("{}", name);
        idx + 1
    }

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_idx: u8 = self.code[offset + 1].into();
        print!("{:-16}{:4} '", name, &constant_idx);
        offset + 2
    }

    fn jump_instruction(&self, name: &str, sign: i16, offset: usize) -> usize {
        let jump = (((self.code[offset + 1] as u16) << 8) | self.code[offset + 2] as u16) as usize;
        let jump_to = if sign == 1 {
            offset + 3 + jump
        } else {
            offset + 3 - jump
        };
        println!("{:-16} {:4} -> {}", name, offset, jump_to);
        offset + 3
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_idx: u8 = self.code[offset + 1].into();
        print!("{:-16}{:4} '", name, &constant_idx);
        print!("{}", self.constants[constant_idx as usize]);
        println!("'");
        offset + 2
    }
}