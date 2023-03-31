//pub type Value = f64;
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

pub fn print_value(val: &Value) {
    match val {
        Value::Bool(n)  => print!("bool: {:?}", n),
        Value::Nil              => print!("nil"),
        Value::Number(n) => print!("number: {:?}", n),
    }
}