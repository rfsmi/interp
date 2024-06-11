use paste::paste;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::{function::Function, thread::Thread, value::Value};

pub enum Object {
    Function(Function),
    Thread(Thread),
}

impl Object {
    pub fn references(&self) -> impl Iterator<Item = &usize> {
        match self {
            Object::Function(f) => f.closure.iter(),
            Object::Thread(t) => t.stack.iter(),
        }
        .filter_map(|value| {
            if let Value::Object(i) = value {
                Some(i)
            } else {
                None
            }
        })
    }

    pub fn references_mut(&mut self) -> impl Iterator<Item = &mut usize> {
        match self {
            Object::Function(f) => f.closure.iter_mut(),
            Object::Thread(t) => t.stack.iter_mut(),
        }
        .filter_map(|value| {
            if let Value::Object(i) = value {
                Some(i)
            } else {
                None
            }
        })
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Object::Function(function) => {
                format!(
                    "function entry:{} params:{} enclosing:{}",
                    function.entry,
                    function.num_params,
                    function.closure.len()
                )
            }
            Object::Thread(_) => format!("thread"),
        };
        write!(f, "{s}")
    }
}
