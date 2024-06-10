use paste::paste;
use std::collections::{HashMap, HashSet};

use crate::{function::Function, object::Object, thread::Thread, value::Value};

pub struct ObjectPool {
    objects: Vec<Object>,
}

pub struct ObjectPoolUpdater {
    pool: ObjectPool,
    mapping: HashMap<usize, usize>,
}

impl ObjectPool {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn allocate(&mut self, object: Object) -> Value {
        let value = Value::Object(self.objects.len());
        self.objects.push(object);
        value
    }

    pub fn compact<'a>(self, roots: impl IntoIterator<Item = &'a Value>) -> ObjectPoolUpdater {
        // Start with the roots
        let mut queue: Vec<usize> = Vec::new();
        for value in roots.into_iter() {
            if let &Value::Object(i) = value {
                queue.push(i);
            }
        }
        // Recursively visit their references
        let mut keep = HashSet::new();
        while let Some(i) = queue.pop() {
            if keep.insert(i) {
                queue.extend(self.objects[i].references())
            }
        }
        // Build a list from the visited objects
        let mut mapping = HashMap::new();
        let mut objects = Vec::new();
        for (i, o) in self.objects.into_iter().enumerate() {
            if keep.contains(&i) {
                mapping.insert(i, objects.len());
                objects.push(o);
            }
        }
        // Update each object's references
        for o in &mut objects {
            for i in o.references_mut() {
                *i = *mapping.get(i).unwrap();
            }
        }
        ObjectPoolUpdater {
            pool: ObjectPool { objects },
            mapping,
        }
    }

    pub fn to_string(&self, value: &Value) -> String {
        match value {
            Value::Nil => format!("Nil"),
            Value::Integer(n) => format!("{n}"),
            Value::Object(i) => format!("{}", &self.objects[*i]),
        }
    }
}

impl ObjectPoolUpdater {
    pub fn update_value(&self, value: &mut Value) {
        if let Value::Object(i) = value {
            *i = *self.mapping.get(i).unwrap();
        }
    }

    pub fn into_pool(self) -> ObjectPool {
        self.pool
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
