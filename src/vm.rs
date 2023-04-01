use crate::chunk::{Chunk, OpCode};
use crate::value::{print_value, Value, values_equal};
use crate::compiler::Parser;

pub struct VM {
    pub chunk: Chunk,
    pub ip: usize,
    pub stack: Vec<Value>,
}

#[derive(PartialEq, Debug)]
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

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut parser = Parser::new(source);

        if !parser.compile() {
            return InterpretResult::CompileError;
        }

        self.chunk = parser.chunk;
        self.ip = 0; // or self.chunk.code?

        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {

        loop {
           self.debug_trace_execution();

            let opcode = &self.chunk.code[self.ip];

            match opcode {
                OpCode::OpConstant(index) => {
                    let constant = &self.chunk.constants[*index as usize];
                    print_value(constant);
                    self.stack.push((*constant).clone());
                    print!("\n");
                },

                OpCode::OpNegate => match self.stack.get(self.stack.len() - 1).expect("Failed to peek") {
                    Value::Number(val) => {
                        let neg_val = -*val;
                        self.stack.pop();
                        self.stack.push(Value::Number(neg_val));
                    },
                    _ => {
                        return self.runtime_error("Operand must be a number.");
                    }
                },

                OpCode::OpNil => self.stack.push(Value::Nil),
                OpCode::OpTrue => self.stack.push(Value::Bool(true)),
                OpCode::OpFalse => self.stack.push(Value::Bool(false)),

                OpCode::OpEqual => {
                    let val1 = self.stack.pop().expect("Empty stack");
                    let val2 = self.stack.pop().expect("Empty stack");
                    self.stack.push(Value::Bool(values_equal(val1, val2)));
                }

                OpCode::OpGreater =>  self.binary_op_bool(|a, b| a > b),
                OpCode::OpLess => self.binary_op_bool(|a, b| a < b),

                OpCode::OpAdd => {
                    let b = self.stack.get(self.stack.len() - 1).expect("Failed to get");
                    let a = self.stack.get(self.stack.len() - 2).expect("Failed to get");
                    match (b, a) {
                        (Value::Number(_), Value::Number(_)) => { self.binary_op(|a, b| a + b) },
                        (Value::Obj(_), Value::Obj(_)) => { self.concatenate().expect("Operands must be two strings."); },
                        _ => return self.runtime_error("Operands must be two numbers or two strings."),
                    }
                },
                OpCode::OpSubtract => self.binary_op(|a, b| a - b),
                OpCode::OpMultiply => self.binary_op(|a, b| a * b),
                OpCode::OpDivide => self.binary_op(|a, b| a / b),
                OpCode::OpNot => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(self.is_falsey(val)))
                },

                OpCode::OpPrint => {
                    print_value(&self.stack.pop().expect("Empty stack"));
                    println!();
                    return InterpretResult::Ok;
                },

                OpCode::OpReturn => { return InterpretResult::Ok; },
            }
            self.ip += 1;
        }
    }

    fn concatenate(&mut self)  -> Result<(), InterpretResult> {
        let b = self.stack.pop().expect("Empty stack");
        let a = self.stack.pop().expect("Empty stack");
        match (b, a) {
            (Value::Obj(b), Value::Obj(a)) => {
                self.stack.push(Value::Obj(a + &b));
                Ok(())
            }
            _ => Err(self.runtime_error("Operands must be two strings."))
        }
    }

    pub  fn binary_op(&mut self, f: fn(f64, f64) -> f64) {
        let b = self.stack.pop().expect("Empty stack");
        let a = self.stack.pop().expect("Empty stack");

        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let result = f(a, b);
                self.stack.push(Value::Number(result));
            }
            _ => {
                self.runtime_error("Operands must be two numbers or two strings");
            }
        }
    }

    pub  fn binary_op_bool(&mut self, f: fn(f64, f64) -> bool) {
        let b = self.stack.pop().expect("Empty stack");
        let a = self.stack.pop().expect("Empty stack");

        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                let result = f(a, b);
                self.stack.push(Value::Bool(result));
            }
            _ => {
                self.runtime_error("Operands must be boolean.");
            }
        }
    }

    fn is_falsey(&self, val: Value) -> bool {
        match val {
            Value::Bool(b) => !b,
            Value::Nil => true,
            _ => false
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

    fn runtime_error(&self, msg: &str) -> InterpretResult {
        eprintln!("{}", msg);

        let instruction = self.ip - 1;
        let line = self.chunk.lines[instruction];
        eprintln!("[line {}] in script", line);
        InterpretResult::RuntimeError
    }
}