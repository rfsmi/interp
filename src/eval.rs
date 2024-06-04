use std::{collections::HashMap, fmt::Display};

struct Closure {
    start: usize,
}

#[derive(Clone, Copy)]
enum Value {
    Nil,
    Integer(i64),
    Closure { start: usize },
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "()"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::Closure { start } => write!(f, "closure {{ {start} }}"),
        }
    }
}

impl Value {
    fn integer(self) -> i64 {
        match self {
            Value::Integer(integer) => integer,
            _ => panic!("expected integer"),
        }
    }

    fn closure(self) -> usize {
        match self {
            Value::Closure { start } => start,
            _ => panic!("expected integer"),
        }
    }
}

enum Expr {
    Store { name: String },
    Load { name: String },
    Literal { integer: i64 },
    Enclose { start: usize },
    Return,
    Call,
    Add,
}

struct Frame {
    addr: usize,
    names: HashMap<String, Value>,
}

pub struct Thread<'pool> {
    pub debug: bool,
    exprs: &'pool Vec<Expr>,
    stack: Vec<Value>,
    frames: Vec<Frame>,
}

impl<'pool> Thread<'pool> {
    pub fn new(exprs: &'pool Vec<Expr>, start: usize) -> Self {
        Self {
            exprs,
            debug: false,
            stack: vec![],
            frames: vec![Frame {
                addr: start,
                names: [].into(),
            }],
        }
    }

    pub fn done(&self) -> bool {
        self.frames.is_empty()
    }

    fn load(&self, name: &str) -> Value {
        let value = self.frames.last().unwrap().names.get(name).unwrap().clone();
        if self.debug {
            println!(" - load {name} ({value})");
        }
        value
    }

    fn store(&mut self, name: String, value: Value) {
        if self.debug {
            println!(" - store {name} = {value}");
        }
        self.frames.last_mut().unwrap().names.insert(name, value);
    }

    fn pop(&mut self) -> Value {
        let value = self.stack.pop().unwrap();
        if self.debug {
            println!(" - pop {value}");
        }
        value
    }

    fn push(&mut self, value: Value) {
        if self.debug {
            println!(" - push {value}");
        }
        self.stack.push(value);
    }

    pub fn step(&mut self) {
        let Some(expr) = self.frames.last_mut().map(|f| {
            let addr = f.addr;
            f.addr += 1;
            &self.exprs[addr]
        }) else {
            return;
        };
        if self.debug {
            match expr {
                Expr::Store { .. } => println!("Store"),
                Expr::Load { .. } => println!("Load"),
                Expr::Literal { .. } => println!("Literal"),
                Expr::Enclose { .. } => println!("Enclose"),
                Expr::Return => println!("Return"),
                Expr::Call => println!("Call"),
                Expr::Add => println!("Add"),
            }
        }
        match expr {
            Expr::Store { name } => {
                let value = self.pop();
                self.store(name.clone(), value);
            }
            Expr::Load { name } => {
                let value = self.load(name);
                self.push(value);
            }
            Expr::Literal { integer } => {
                self.push(Value::Integer(*integer));
            }
            Expr::Add => {
                let a = self.pop().integer();
                let b = self.pop().integer();
                self.push(Value::Integer(a + b));
            }
            Expr::Enclose { start } => {
                self.push(Value::Closure { start: *start });
            }
            Expr::Return => {
                self.frames.pop().unwrap();
            }
            Expr::Call => {
                let frame = Frame {
                    addr: self.pop().closure(),
                    names: [].into(),
                };
                self.frames.push(frame);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        let exprs = vec![
            Expr::Literal { integer: 2 },
            Expr::Literal { integer: 2 },
            Expr::Add,
            Expr::Return,
            Expr::Enclose { start: 0 },
            Expr::Store { name: "c".into() },
            Expr::Load { name: "c".into() },
            Expr::Call,
            Expr::Load { name: "c".into() },
            Expr::Call,
            Expr::Add,
            Expr::Return,
        ];
        let mut thread = Thread::new(&exprs, 4);
        thread.debug = true;
        while !thread.done() {
            thread.step();
        }
    }
}
