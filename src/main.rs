use crate::chunk::{Chunk, OpCode};

mod chunk;
mod value;

fn main() {

    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write_chunk(OpCode::OpConstant(constant.try_into().unwrap()));
    chunk.write_chunk(OpCode::OpReturn);

    chunk.disassemble_chunk("test chunk");
}
