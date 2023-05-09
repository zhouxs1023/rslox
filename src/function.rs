use std::fmt;
use crate::chunk::*;

#[derive(Debug)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
}

impl Function {
    pub fn new() -> Self {
        Function {
            arity: 0,
            chunk: Chunk::new(),
            name: "".to_string(),
        }
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        panic!("Comparing Ord of two functions")
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Self {
            arity: self.arity,
            chunk: self.chunk.clone(),
            name: self.name.clone(),
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.name.is_empty() {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", self.name)
        }
    }
}