use crate::chunk::{Chunk, OpCode};

mod chunk;

fn main() {

    let mut chunk = Chunk::new();
    chunk.write_chunk(OpCode::OpReturn);

    chunk.disassemble_chunk("test chunk");
}
