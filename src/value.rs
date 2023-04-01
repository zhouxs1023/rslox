//pub type Value = f64;
static ERR_MARGIN: f64 = f64::EPSILON;
#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(String),
}

pub fn values_equal(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => (a - b).abs() < ERR_MARGIN,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        (Value::Obj(a), Value::Obj(b)) => a == b,
        _ => false,
    }
}

pub fn print_value(val: &Value) {
    match val {
        Value::Bool(n)  => print!("bool: {:?}", n),
        Value::Nil              => print!("nil"),
        Value::Number(n) => print!("number: {:?}", n),
        Value::Obj(st) => print!("string: {:?}", st),
    }
}