use std::rc::Rc;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use crate::chunk::{Chunk, OpCode};
use crate::value::{print_value, Value, values_equal};
use crate::compiler::Parser;

pub struct VM {
    pub chunk: Rc<Chunk>,
    pub ip: usize,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
}

#[derive(PartialEq, Debug)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {

    pub fn new() -> Self {
        Self {
            chunk: Rc::new(Chunk::new()),
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut parser = Parser::new(source);

        if !parser.compile() {
            return InterpretResult::CompileError;
        }

        self.chunk = Rc::new(parser.chunk);
        self.ip = 0; // or self.chunk.code?

        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {

        loop {
           self.debug_trace_execution();

            let opcode: OpCode = self.read_opcode();

            match opcode {
                OpCode::OpConstant => {
                    let constant = self.read_constant().clone();
                    self.stack.push(constant);
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
                OpCode::OpPop => { self.stack.pop().expect("Empty stack");},

                OpCode::OpDefineGlobal => {
                    let name = self.read_constant().clone();
                    if let Value::ObjString(s) = name {
                        let val = self.stack.pop().expect("Empty stack");
                        self.globals.insert(s, val);
                    } else {
                        panic!("Unable to read global variable");
                    }
                },

                OpCode::OpGetLocal => {
                    let slot: u8 = self.read_opcode().into();
                    self.stack.push(self.stack[slot as usize].clone());
                },

                OpCode::OpSetLocal => {
                    let slot: u8 = self.read_opcode().into();
                    self.stack[slot as usize] = self.peek(0).clone();
                },

                OpCode::OpGetGlobal => {
                    if let Value::ObjString(s) = self.read_constant().clone() {
                        if let Some(v) = self.globals.get(&s) {
                            self.stack.push(v.clone());
                        } else {
                            return self.runtime_error("Undefined variable .");
                        }
                    } else {
                        panic!("Unable to read constant from table.");
                    }
                },

                OpCode::OpSetGlobal => {
                    if let Value::ObjString(s) = self.read_constant().clone() {
                        let val = self.peek(0).clone();
                        if let Entry::Occupied(mut o) = self.globals.entry(s.clone()) {
                            *o.get_mut() = val;
                        } else {
                            return self.runtime_error("Undefined variable ");
                        }
                    } else {
                        panic!("Unable to read constant from table.");
                    }
                },

                OpCode::OpEqual => {
                    let val1 = self.stack.pop().expect("Empty stack");
                    let val2 = self.stack.pop().expect("Empty stack");
                    self.stack.push(Value::Bool(values_equal(val1, val2)));
                },

                OpCode::OpGreater =>  self.binary_op_bool(|a, b| a > b),
                OpCode::OpLess => self.binary_op_bool(|a, b| a < b),

                OpCode::OpAdd => {
                    match (self.peek(0), self.peek(1)) {
                        (Value::Number(_), Value::Number(_)) => { self.binary_op(|a, b| a + b) },
                        (Value::ObjString(_), Value::ObjString(_)) => { self.concatenate().expect("Operands must be two strings."); },
                        _ => return self.runtime_error("Operands must be two numbers or two strings."),
                    }
                },
                OpCode::OpSubtract => self.binary_op(|a, b| a - b),
                OpCode::OpMultiply => self.binary_op(|a, b| a * b),
                OpCode::OpDivide => self.binary_op(|a, b| a / b),
                OpCode::OpNot => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(self.is_falsey(&val)))
                },

                OpCode::OpPrint => {
                    print_value(&self.stack.pop().expect("Empty stack"));
                    println!();
                    return InterpretResult::Ok;
                },

                OpCode::OpJumpIfFalse => {
                    let offset = self.read_short();
                    if self.is_falsey(self.peek(0)) {
                        self.ip += offset;
                    }
                },

                OpCode::OpJump => {
                    let offset = self.read_short();
                    self.ip += offset;
                }

                OpCode::OpReturn => { return InterpretResult::Ok; },
            }
        }
    }

    fn concatenate(&mut self)  -> Result<(), InterpretResult> {
        let b = self.stack.pop().expect("Empty stack");
        let a = self.stack.pop().expect("Empty stack");
        match (b, a) {
            (Value::ObjString(b), Value::ObjString(a)) => {
                self.stack.push(Value::ObjString(a + &b));
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

    fn peek(&self, distance: usize) -> &Value {
        return self
            .stack
            .get(self.stack.len() - 1 - distance)
            .expect("Failed to peek");
    }

    fn read_short(&mut self) -> usize {
        self.ip += 2;
        (((self.chunk.code[self.ip - 2] as u16) << 8) | self.chunk.code[self.ip - 1] as u16) as usize
    }

    fn read_opcode(&mut self) -> OpCode {
        let val = self.chunk.read_byte(self.ip).into();
        self.ip += 1;
        val
    }

    fn read_constant(&mut self) -> &Value {
        let idx = self.chunk.read_byte(self.ip) as usize;
        self.ip += 1;
        self.chunk.get_constant(idx)
    }

    fn is_falsey(&self, val: &Value) -> bool {
        match *val {
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