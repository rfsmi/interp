use paste::paste;
use std::collections::{HashMap, HashSet};

pub enum Value {
    Nil,
    Integer(i64),
    Object(usize),
}

pub struct Closure {
    pub values: Vec<Value>,
}

pub struct Stack {}

enum Object {
    Closure(Closure),
    Stack(Stack),
}

pub struct ObjectPool {
    objects: Vec<Object>,
}

pub struct ObjectPoolUpdater {
    pool: ObjectPool,
    mapping: HashMap<usize, usize>,
}

impl Object {
    fn references(&self, mut f: impl FnMut(&usize)) {
        match self {
            Object::Closure(closure) => {
                for value in &closure.values {
                    if let Value::Object(i) = value {
                        f(i);
                    }
                }
            }
            Object::Stack(_) => todo!(),
        }
    }

    fn references_mut(&mut self, mut f: impl FnMut(&mut usize)) {
        match self {
            Object::Closure(closure) => {
                for value in &mut closure.values {
                    if let Value::Object(i) = value {
                        f(i);
                    }
                }
            }
            Object::Stack(_) => todo!(),
        }
    }
}

impl ObjectPool {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
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
                self.objects[i].references(|&j| queue.push(j));
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
            o.references_mut(|i| *i = *mapping.get(i).unwrap());
        }
        ObjectPoolUpdater {
            pool: ObjectPool { objects },
            mapping,
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
                pub fn [<$($kind)*:lower>](&'pool self, value: &Value) -> &'pool $($kind)* {
                    if let &Value::Object(i) = value {
                        if let Object::$($kind)*(kind) = &self.objects[i] {
                            return kind;
                        }
                    }
                    panic!(std::stringify!(value is not a [<$($kind)*:lower>]));
                }
                pub fn [<$($kind)*:lower _mut>](&'pool mut self, value: &Value) -> &'pool mut $($kind)* {
                    if let &Value::Object(i) = value {
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

decl_getters!(Closure);
decl_getters!(Stack);
