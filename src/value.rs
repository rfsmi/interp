#[derive(Clone, Copy)]
pub enum Value {
    Nil,
    Integer(i64),
    Object(usize),
}

impl Value {
    pub fn integer(&self) -> i64 {
        if let Value::Integer(n) = self {
            return *n;
        }
        panic!("value is not an integer");
    }
}
