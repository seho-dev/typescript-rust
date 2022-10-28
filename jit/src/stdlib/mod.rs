use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use crate::{builder::Builder, value::Class, Value};

#[derive(Debug)]
pub struct Array {
    pub(crate) data: Vec<Arc<Value>>,
    me: Weak<Mutex<Self>>,
}

unsafe extern "C" fn array_iterator(arr: *const Value) -> *const Value {
    let val = match &*arr {
        // Value::Array(_arr) => Value::Class(ArrayIterator::new()),
        Value::Class(arr) => Value::Class(ArrayIterator::new(arr.clone())),
        _ => Value::Null,
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
    pub fn register(module: &mut Builder) {
        extern "C" fn array_new() -> *const Value {
            #[cfg(feature = "trace")]
            log::trace!("!! new array !!");
            // Arc::into_raw(Arc::new(Value::Array(Vec::new())))
            Arc::into_raw(Arc::new(Value::Class(Array::new())))
        }

        unsafe extern "C" fn array_push(arr: *mut Value, v: *mut Value) -> *const Value {
            #[cfg(feature = "trace")]
            log::trace!("!! pushing value !! {:?} {:?}", *arr, *v);

            match &mut *arr {
                Value::Array(ref mut a) => {
                    a.push(Arc::from_raw(v));
                }
                Value::Class(a) => {
                    let mut ag = a.lock().unwrap();
                    if let Some(arr) = ag.as_any().downcast_mut::<Array>() {
                        arr.data.push(Arc::from_raw(v));
                    }
                }
                _ => {
                    log::warn!("we can not push");
                }
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
    fn set(&mut self, name: Arc<Value>, val: Arc<Value>) {
        #[cfg(feature = "trace")]
        log::trace!("array-class set {:?}", name);

        match &*name {
            Value::Number(n) => {
                let idx = *n as usize;

                if idx < self.data.len() as _ {
                    self.data[idx] = val;
                }
            }
            _ => {}
        }
    }

    fn get(&self, name: Arc<Value>) -> Arc<Value> {
        #[cfg(feature = "trace")]
        log::trace!("array-class get {:?}", name);

        match &*name {
            Value::Number(n) => {
                let idx = *n as usize;

                if idx < self.data.len() as _ {
                    self.data[idx].clone()
                } else {
                    Arc::new(Value::Null)
                }
            }
            Value::Str(name) => match name.as_str() {
                "length" => Arc::new(Value::Number(self.data.len() as f64)),
                "@iterator" => Arc::new(Value::Method {
                    class: self.me.upgrade().unwrap(),
                    func: array_iterator as _,
                }),
                _ => Arc::new(Value::Null),
            },
            _ => Arc::new(Value::Null),
        }
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct ArrayIterator {
    array: Arc<Mutex<dyn Class>>,
    step: usize,
    me: Weak<Mutex<Self>>,
}

unsafe extern "C" fn array_iterator_next(arr_it: *const Value) -> *const Value {
    if let Value::Class(arr_it) = &*arr_it {
        let mut arr_it = arr_it.lock().unwrap();
        if let Some(arr_it) = arr_it.as_any().downcast_mut::<ArrayIterator>() {
            return Arc::into_raw(arr_it.next());
        }
    }

    let mut obj = HashMap::new();
    obj.insert("value".to_owned(), Arc::new(Value::Null));
    obj.insert("done".to_owned(), Arc::new(Value::Boolean(true)));
    Arc::into_raw(Arc::new(Value::Object(obj)))
}

impl ArrayIterator {
    pub fn new(array: Arc<Mutex<dyn Class>>) -> Arc<Mutex<dyn Class>> {
        Arc::new_cyclic(|me| {
            Mutex::new(ArrayIterator {
                array,
                step: 0,
                me: me.clone(),
            })
        })
    }

    pub fn register(module: &mut Builder) {
        module.add_fn("__array_iterator_next", array_iterator_next as _, 1);
    }

    pub fn next(&mut self) -> Arc<Value> {
        let mut obj = HashMap::new();
        obj.insert("value".to_owned(), Arc::new(Value::Null));
        obj.insert("done".to_owned(), Arc::new(Value::Boolean(true)));

        let mut arr = self.array.lock().unwrap();
        if let Some(arr) = arr.as_any().downcast_ref::<Array>() {
            if self.step < arr.data.len() {
                let sample = arr.data[self.step].clone();
                self.step += 1;
                obj.insert("value".to_owned(), sample);
                obj.insert("done".to_owned(), Arc::new(Value::Boolean(false)));
            }
        }
        
        Arc::new(Value::Object(obj))
    }
}

impl Class for ArrayIterator {
    fn set(&mut self, _name: Arc<Value>, _val: Arc<Value>) {}

    fn get(&self, name: Arc<Value>) -> Arc<Value> {
        match &*name {
            Value::Str(name) => match name.as_str() {
                "next" => {
                    return Arc::new(Value::Method {
                        class: self.me.upgrade().unwrap(),
                        func: array_iterator_next as _,
                    });
                }
                _ => {}
            },
            _ => {}
        }

        Arc::new(Value::Null)
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
