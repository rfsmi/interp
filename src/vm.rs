use crate::{
    function::Function,
    object::Object,
    pool::ObjectPool,
    thread::{Frame, Thread},
    value::Value,
};

#[derive(Clone, Copy)]
enum Expr {
    Load {
        i: usize,
    },
    Literal {
        integer: i64,
    },
    Function {
        entry: usize,
        closure_len: u32,
        num_params: u32,
    },
    Return,
    Call {
        num_args: u32,
    },
    Add,
}

struct VM {
    exprs: Vec<Expr>,
    pool: ObjectPool,
    thread: Value,
    pub debug: bool,
}

impl VM {
    pub fn new(exprs: Vec<Expr>, entry: usize) -> Self {
        let mut pool = ObjectPool::new();
        let thread = pool.allocate(Object::Thread(Thread::new(entry)));
        Self {
            exprs,
            pool,
            thread,
            debug: false,
        }
    }

    fn thread_mut(&mut self) -> &mut Thread {
        self.pool.thread_mut(self.thread)
    }

    fn thread(&self) -> &Thread {
        self.pool.thread(self.thread)
    }

    pub fn running(&self) -> bool {
        !self.thread().frames.is_empty()
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
                Expr::Function {
                    entry,
                    closure_len,
                    num_params,
                } => {
                    println!("function addr:{entry} params:{num_params} closure:{closure_len}")
                }
                Expr::Return => println!("return"),
                Expr::Call { num_args } => println!("call args:{num_args}"),
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
                num_params,
            } => {
                let closure = Object::Function(Function {
                    entry: first_expr,
                    num_params,
                    closure: self.thread_mut().pop_n(closure_len as usize),
                });
                let value = self.pool.allocate(closure);
                self.thread_mut().push(value);
            }
            Expr::Return => {
                self.thread_mut().ret();
            }
            Expr::Call { num_args } => {
                let value = self.thread_mut().pop();
                let function = self.pool.function(value).clone();
                if num_args != function.num_params {
                    panic!(
                        "function called with {} args but expected {}",
                        num_args, function.num_params
                    );
                }
                self.thread_mut().call(function);
            }
        }
        if self.debug && self.running() {
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
        /*
           adder := (x) => {
               (y) => {
                   x + y
               }
           }
           adder(2)(1)
        */
        let exprs = vec![
            Expr::Load { i: 0 }, // y
            Expr::Load { i: 1 }, // x (enclosed)
            Expr::Add,
            Expr::Return,
            // adder
            Expr::Load { i: 0 },
            Expr::Function {
                entry: 0,
                closure_len: 1,
                num_params: 1,
            },
            Expr::Return,
            Expr::Literal { integer: 1 },
            Expr::Literal { integer: 2 },
            Expr::Function {
                entry: 4,
                closure_len: 0,
                num_params: 1,
            },
            Expr::Call { num_args: 1 },
            Expr::Call { num_args: 1 },
            Expr::Return,
        ];
        let mut vm = VM::new(exprs, 7);
        vm.debug = true;
        while vm.running() {
            vm.step();
        }
    }
}
