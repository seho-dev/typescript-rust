use std::{collections::HashMap, sync::{Arc, Weak, Mutex}};

use crate::{value::Class, Module, Value};

#[derive(Debug)]
pub struct Array {
    data: Vec<Arc<Value>>,
    me: Weak<Mutex<Self>>,
}

unsafe extern "C" fn array_iterator(arr: *const Value) -> *const Value {
    let val = match &*arr {
        Value::Array(_arr) => {
            Value::Class(ArrayIterator::new())
        }
        Value::Class(_arr) => {
            Value::Class(ArrayIterator::new())
        }
        _ => {
            Value::Null
        }
    };

    Arc::into_raw(Arc::new(val))
}

impl Array {
    pub fn new() -> Arc<Mutex<dyn Class>> {
        Arc::new_cyclic(|me| {
            Mutex::new(Array {
                data: Vec::new(),
                me: me.clone(),
            })
        })
    }
    pub fn register(module: &mut Module) {
        extern "C" fn array_new() -> *const Value {
            #[cfg(feature = "trace")]
            log::trace!("!! new array !!");
            Arc::into_raw(Arc::new(Value::Array(Vec::new())))
        }

        unsafe extern "C" fn array_push(arr: *mut Value, v: *mut Value) -> *const Value {
            #[cfg(feature = "trace")]
            log::trace!("!! pushing value !! {:?} {:?}", *arr, *v);

            if let Value::Array(ref mut a) = *arr {
                a.push(Arc::from_raw(v));
            }

            &Value::Null as *const _
        }

        module.add_fn("__array_new", array_new as _, 0);
        module.add_fn("__array_push", array_push as _, 2);
        module.add_fn("__array_iterator", array_iterator as _, 1);

        ArrayIterator::register(module);
    }
}

impl Class for Array {
    fn set_attribute(&mut self, name: &str, val: Arc<Value>) {}

    fn get_attribute(&self, name: &str) -> Arc<Value> {
        #[cfg(feature = "trace")]
        log::trace!("rust-class get {}", name);

        match name {
            "length" => Arc::new(Value::Number(self.data.len() as f64)),
            "@iterator" => Arc::new(Value::Method { class: self.me.upgrade().unwrap(), func: array_iterator as _ }),
            _ => Arc::new(Value::Null),
        }
    }

    fn set_index(&mut self, idx: u32, val: Arc<Value>) {
        if idx < self.data.len() as _ {
            self.data[idx as usize] = val;
        }
    }

    fn get_index(&self, idx: u32) -> Arc<Value> {
        if idx < self.data.len() as _ {
            self.data[idx as usize].clone()
        } else {
            Arc::new(Value::Null)
        }
    }
}

#[derive(Debug)]
pub struct ArrayIterator {
    step: usize,
    me: Weak<Mutex<Self>>,
}

unsafe extern "C" fn array_iterator_next(arr: *const Value) -> *const Value {
    let mut obj = HashMap::new();
    obj.insert("value".to_owned(), Arc::new(Value::Null));
    obj.insert("done".to_owned(), Arc::new(Value::Boolean(true)));

    Arc::into_raw(Arc::new(Value::Object(obj)))
}

impl ArrayIterator {
    pub fn new() -> Arc<Mutex<dyn Class>> {
        Arc::new_cyclic(|me| {
            Mutex::new(ArrayIterator {
                step: 0,
                me: me.clone(),
            })
        })
    }

    pub fn register(module: &mut Module) {
        module.add_fn("__array_iterator_next", array_iterator_next as _, 1);
    }
}

impl Class for ArrayIterator {
    fn set_attribute(&mut self, name: &str, val: Arc<Value>) {}

    fn get_attribute(&self, name: &str) -> Arc<Value> {
        match name {
            "next" => Arc::new(Value::Method { class: self.me.upgrade().unwrap(), func: array_iterator_next as _ }),
            _ => {
                Arc::new(Value::Null)
            }
        }
    }

    fn set_index(&mut self, idx: u32, val: Arc<Value>) {}

    fn get_index(&self, idx: u32) -> Arc<Value> {
        Arc::new(Value::Null)
    }
}
