use crate::value::Value;

#[derive(Clone)]
pub struct Function {
    pub entry: usize,
    pub closure: Vec<Value>,
}
