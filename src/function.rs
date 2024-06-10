use crate::value::Value;

#[derive(Clone)]
pub struct Function {
    pub entry: usize,
    pub num_params: u32,
    pub closure: Vec<Value>,
}
