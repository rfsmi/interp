use paste::paste;
use std::collections::{HashMap, HashSet};

use crate::{function::Function, object::Object, thread::Thread, value::Value};

pub struct ObjectPool {
    objects: Vec<Object>,
}

impl ObjectPool {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn allocate(&mut self, object: Object) -> Value {
        let value = Value::Object(self.objects.len());
        self.objects.push(object);
        value
    }

    pub fn compact(&mut self, root: Value) -> Value {
        let Value::Object(root) = root else {
            self.objects.clear();
            return root;
        };
        let mut queue = vec![root];
        // Recursively visit each reference
        let mut keep = HashSet::new();
        while let Some(i) = queue.pop() {
            if keep.insert(i) {
                queue.extend(self.objects[i].references())
            }
        }
        // Build a mapping from old index to new index while copying objects
        // from the old pool to the new one.
        let mut mapping = HashMap::new();
        let objects = std::mem::take(&mut self.objects);
        for (i, o) in objects.into_iter().enumerate() {
            if keep.contains(&i) {
                mapping.insert(i, self.objects.len());
                self.objects.push(o);
            }
        }
        // Update each object's references
        for o in &mut self.objects {
            for i in o.references_mut() {
                *i = *mapping.get(i).unwrap();
            }
        }
        Value::Object(*mapping.get(&root).unwrap())
    }

    pub fn to_string(&self, value: &Value) -> String {
        match value {
            Value::Nil => format!("Nil"),
            Value::Integer(n) => format!("{n}"),
            Value::Object(i) => format!("{}", &self.objects[*i]),
        }
    }
}

macro_rules! decl_getters {
    ($($kind:tt)*) => {
        paste!{
            impl<'pool> ObjectPool {
                pub fn [<$($kind)*:lower>](&'pool self, value: Value) -> &'pool $($kind)* {
                    if let Value::Object(i) = value {
                        if let Object::$($kind)*(kind) = &self.objects[i] {
                            return kind;
                        }
                    }
                    panic!(std::stringify!(value is not a [<$($kind)*:lower>]));
                }
                pub fn [<$($kind)*:lower _mut>](&'pool mut self, value: Value) -> &'pool mut $($kind)* {
                    if let Value::Object(i) = value {
                        if let Object::$($kind)*(kind) = &mut self.objects[i] {
                            return kind;
                        }
                    }
                    panic!(std::stringify!(value is not a [<$($kind)*:lower>]));
                }
            }
        }
    };
}

decl_getters!(Function);
decl_getters!(Thread);
