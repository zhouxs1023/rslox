use crate::chunk::{Chunk, OpCode};

mod chunk;
mod value;

fn main() {

    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write_chunk(OpCode::OpConstant(u8::try_from(constant).unwrap()), 123);
    chunk.write_chunk(OpCode::OpReturn, 123);

    chunk.disassemble_chunk("test chunk");
}
