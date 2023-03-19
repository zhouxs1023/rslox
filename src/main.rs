use crate::chunk::{Chunk, OpCode};
use crate::vm::VM;

mod chunk;
mod value;
mod vm;

fn main() {

    let mut vm = VM::new();
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write_chunk(OpCode::OpConstant(u8::try_from(constant).unwrap()), 123);

    let constant = chunk.add_constant(3.4);
    chunk.write_chunk(OpCode::OpConstant(u8::try_from(constant).unwrap()), 123);

    chunk.write_chunk(OpCode::OpNegate, 123);
    chunk.write_chunk(OpCode::OpReturn, 123);

    // chunk.disassemble_chunk("test chunk");

    vm.interpret(chunk);
}
