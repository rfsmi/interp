use crate::{
    function::Function,
    object::Object,
    pool::ObjectPool,
    thread::{Frame, Thread},
    value::Value,
};

#[derive(Clone, Copy)]
enum Expr {
    Load { i: usize },
    Literal { integer: i64 },
    Function { entry: usize, closure_len: u32 },
    Return,
    Call,
    Add,
}

struct VM {
    exprs: Vec<Expr>,
    pool: ObjectPool,
    threads: Vec<Value>,
    pub debug: bool,
}

impl VM {
    pub fn new(exprs: Vec<Expr>) -> Self {
        Self {
            exprs,
            pool: ObjectPool::new(),
            threads: Vec::new(),
            debug: false,
        }
    }

    pub fn new_thread(&mut self, target: usize) {
        let thread = Object::Thread(Thread::new(target));
        self.threads.push(self.pool.allocate(thread));
    }

    fn thread_mut(&mut self) -> &mut Thread {
        self.pool.thread_mut(self.threads[0])
    }

    fn thread(&self) -> &Thread {
        self.pool.thread(self.threads[0])
    }

    pub fn running(&self) -> bool {
        !self.threads.is_empty()
    }

    pub fn step(&mut self) {
        let Some(addr) = self.thread_mut().advance() else {
            return;
        };
        let expr = self.exprs[addr];
        if self.debug {
            print!("{addr:04} -> ");
            match expr {
                Expr::Load { i } => println!("load {i}"),
                Expr::Literal { integer } => println!("literal {integer}"),
                Expr::Function { entry, closure_len } => {
                    println!("function addr:{entry} closure:{closure_len}")
                }
                Expr::Return => println!("return"),
                Expr::Call => println!("call"),
                Expr::Add => println!("add"),
            }
        }
        match expr {
            Expr::Load { i } => {
                let value = self.thread().get(i);
                self.thread_mut().push(value);
            }
            Expr::Literal { integer } => {
                self.thread_mut().push(Value::Integer(integer));
            }
            Expr::Add => {
                let thread = self.thread_mut();
                let a = thread.pop().integer();
                let b = thread.pop().integer();
                thread.push(Value::Integer(a + b));
            }
            Expr::Function {
                entry: first_expr,
                closure_len,
            } => {
                let closure = Object::Function(Function {
                    entry: first_expr,
                    closure: self.thread_mut().pop_n(closure_len as usize),
                });
                let value = self.pool.allocate(closure);
                self.thread_mut().push(value);
            }
            Expr::Return => {
                self.thread_mut().ret();
                if self.thread().done() {
                    self.threads.remove(0);
                    return;
                }
            }
            Expr::Call => {
                let value = self.thread_mut().pop();
                let function = self.pool.function(value).clone();
                self.thread_mut().call(function);
            }
        }
        if self.debug {
            let frame = self.thread().frames.last().unwrap();
            println!("stack (+{}):", frame.stack_offset);
            for i in (frame.stack_offset..self.thread().stack.len()).rev() {
                let value = &self.thread().stack[i];
                println!(
                    "{:>4}| {}",
                    i - frame.stack_offset,
                    self.pool.to_string(value)
                );
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
            Expr::Add,
            Expr::Return,
            Expr::Literal { integer: 1 },
            Expr::Literal { integer: 2 },
            Expr::Function {
                entry: 0,
                closure_len: 2,
            },
            Expr::Load { i: 0 },
            Expr::Call,
            Expr::Load { i: 0 },
            Expr::Call,
            Expr::Add,
            Expr::Return,
        ];
        let mut vm = VM::new(exprs);
        vm.debug = true;
        vm.new_thread(2);
        while vm.running() {
            vm.step();
        }
    }
}
