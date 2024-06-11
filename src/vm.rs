use crate::{function::Function, object::Object, pool::ObjectPool, thread::Thread, value::Value};

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
    Add,
    Sub,
    BranchIfNotZero {
        target: usize,
    },
    Branch {
        target: usize,
    },
    Call {
        num_args: u32,
    },
    Return,
}

struct VM {
    pool: ObjectPool,
    pub debug: bool,
}

impl VM {
    pub fn new() -> Self {
        Self {
            pool: ObjectPool::new(),
            debug: false,
        }
    }

    fn step(&mut self, expr: Expr, thread: Value) -> Option<Value> {
        match expr {
            Expr::Load { i } => {
                let value = self.pool.thread(thread).get(i);
                self.pool.thread_mut(thread).push(value);
            }
            Expr::Literal { integer } => {
                self.pool.thread_mut(thread).push(Value::Integer(integer));
            }
            Expr::Add => {
                let thread = self.pool.thread_mut(thread);
                let b = thread.pop().integer();
                let a = thread.pop().integer();
                thread.push(Value::Integer(a + b));
            }
            Expr::Sub => {
                let thread = self.pool.thread_mut(thread);
                let b = thread.pop().integer();
                let a = thread.pop().integer();
                thread.push(Value::Integer(a - b));
            }
            Expr::Function {
                entry: first_expr,
                closure_len,
                num_params,
            } => {
                let closure = Object::Function(Function {
                    entry: first_expr,
                    num_params,
                    closure: self.pool.thread_mut(thread).pop_n(closure_len as usize),
                });
                let value = self.pool.allocate(closure);
                self.pool.thread_mut(thread).push(value);
            }
            Expr::BranchIfNotZero { target } => {
                if self.pool.thread_mut(thread).pop().integer() != 0 {
                    self.pool.thread_mut(thread).frames.last_mut().unwrap().addr = target;
                }
            }
            Expr::Branch { target } => {
                self.pool.thread_mut(thread).frames.last_mut().unwrap().addr = target;
            }
            Expr::Call { num_args } => {
                let value = self.pool.thread(thread).peek();
                let function = self.pool.function(value).clone();
                if num_args != function.num_params {
                    panic!(
                        "function called with {} args but expected {}",
                        num_args, function.num_params
                    );
                }
                self.pool.thread_mut(thread).call(function);
            }
            Expr::Return => {
                let thread = self.pool.thread_mut(thread);
                thread.ret();
                if thread.done() {
                    return Some(thread.pop());
                }
            }
        }
        None
    }

    fn debug_step(&self, exprs: &[Expr], addr: usize, thread: Value) {
        let frame = self.pool.thread(thread).frames.last().unwrap();
        println!("stack (+{}):", frame.stack_offset);
        for i in (frame.stack_offset..self.pool.thread(thread).stack.len()).rev() {
            let value = &self.pool.thread(thread).stack[i];
            println!(
                "{:>4}| {}",
                i - frame.stack_offset,
                self.pool.to_string(value)
            );
        }
        print!("{addr:04} -> ");
        match exprs[addr] {
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
            Expr::Sub => println!("sub"),
            Expr::BranchIfNotZero { target } => {
                println!("branch if not zero target{target}")
            }
            Expr::Branch { target } => println!("branch target{target}"),
        }
    }

    pub fn exec(&mut self, exprs: &[Expr], entry: usize) -> Value {
        let thread = self.pool.allocate(Object::Thread(Thread::new(entry)));
        loop {
            let addr = self.pool.thread_mut(thread).advance().unwrap();
            if self.debug {
                self.debug_step(exprs, addr, thread);
            }
            if let Some(result) = self.step(exprs[addr], thread) {
                let num_objects = self.pool.len();
                let result = self.pool.compact(result);
                if self.debug {
                    println!("reclaimed {} objects", num_objects - self.pool.len());
                }
                return result;
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fib() {
        /*
           fib := (n) => {
               if      n == 0 { 0 }
               else if n == 1 { 1 }
               else           { fib(n - 2) + fib(n - 1) }
           }
           fib(8)
        */
        let exprs = vec![
            // stack is: 0:n, 1:func
            // fib := (n) => { if i == 0 { return 0; }
            Expr::Load { i: 0 },
            Expr::BranchIfNotZero { target: 4 },
            Expr::Literal { integer: 0 },
            Expr::Return,
            // else if i == 1 { return 1; }
            Expr::Load { i: 0 },
            Expr::Literal { integer: 1 },
            Expr::Sub,
            Expr::BranchIfNotZero { target: 10 },
            Expr::Literal { integer: 1 },
            Expr::Return,
            // else { return fib(n - 2) + fib(n - 1) } }
            Expr::Load { i: 0 },
            Expr::Literal { integer: 2 },
            Expr::Sub,
            Expr::Load { i: 1 },
            Expr::Call { num_args: 1 }, // fib(n - 2)
            Expr::Load { i: 0 },
            Expr::Literal { integer: 1 },
            Expr::Sub,
            Expr::Load { i: 1 },
            Expr::Call { num_args: 1 }, // fib(n - 1)
            Expr::Add,
            Expr::Return,
            // return fib(8)
            Expr::Literal { integer: 8 },
            Expr::Function {
                entry: 0,
                closure_len: 0,
                num_params: 1,
            },
            Expr::Call { num_args: 1 },
            Expr::Return,
        ];
        let mut vm = VM::new();
        vm.debug = true;
        assert_eq!(vm.exec(&exprs, 22).integer(), 21);
    }

    #[test]
    fn test_curry() {
        /*
           adder := (x) => {
               (y) => {
                   x + y
               }
           }
           adder(2)(1)
        */
        let exprs = vec![
            // stack is: 0:y, 1:func, 2:x
            Expr::Load { i: 0 },
            Expr::Load { i: 2 },
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
        let mut vm = VM::new();
        vm.debug = true;
        assert_eq!(vm.exec(&exprs, 7).integer(), 3);
    }
}
