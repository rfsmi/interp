use crate::{function::Function, value::Value};

pub struct Frame {
    pub addr: usize,
    pub stack_offset: usize,
}

pub struct Thread {
    pub stack: Vec<Value>,
    pub frames: Vec<Frame>,
}

impl Thread {
    pub fn new(start: usize) -> Self {
        Self {
            stack: vec![],
            frames: vec![Frame {
                addr: start,
                stack_offset: 0,
            }],
        }
    }

    pub fn advance(&mut self) -> Option<usize> {
        let Some(frame) = self.frames.last_mut() else {
            return None;
        };
        let addr = frame.addr;
        frame.addr += 1;
        Some(addr)
    }

    pub fn done(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn get(&self, i: usize) -> Value {
        let stack_offset = self.frames.last().unwrap().stack_offset;
        self.stack[stack_offset + i]
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    pub fn peek(&self) -> Value {
        *self.stack.last().unwrap()
    }

    pub fn pop_n(&mut self, n: usize) -> Vec<Value> {
        self.stack.split_off(self.stack.len() - n)
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    pub fn ret(&mut self) {
        let frame = self.frames.pop().unwrap();
        let retval = self.pop();
        self.stack.truncate(frame.stack_offset);
        self.push(retval);
    }

    pub fn call(&mut self, function: Function) {
        // Stack frame is laid out as follows (assuming n arguments and m
        // enclosed objects):
        //
        //   n+1+m - closure m
        //   ...
        //   n+1   - closure 0
        //   n     - function object
        //   n-1   - arg n
        //   ...
        //   0     - arg 0
        let stack_size = function.num_params as usize + 1 + function.closure.len();
        self.stack.extend(function.closure);
        let frame = Frame {
            addr: function.entry,
            stack_offset: self.stack.len() - stack_size,
        };
        self.frames.push(frame);
    }
}
