use std::fmt;
use std::fmt::Formatter;

static ERR_MARGIN: f64 = f64::EPSILON;
#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    ObjString(String),
}

pub fn values_equal(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => (a - b).abs() < ERR_MARGIN,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        (Value::ObjString(str1), Value::ObjString(str2)) => str1 == str2,
        _ => false,
    }
}

pub fn print_value(val: &Value) {
    match val {
        Value::Bool(n)  => print!("bool: {:?}", n),
        Value::Nil              => print!("nil"),
        Value::Number(n) => print!("number: {:?}", n),
        Value::ObjString(str) => print!("Objstring: {:?}", str),
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(val) => write!(f, "{}", val),
            Self::Bool(val) => write!(f, "{}", val),
            Self::ObjString(s) => write!(f, "{}", s),
            Self::Nil => write!(f, "nil"),
        }
    }
}