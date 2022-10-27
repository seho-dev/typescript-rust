use std::{sync::Arc, ffi::CStr};

use crate::stdlib::Array;

use super::{value::Value, context::Context};

pub unsafe extern "C" fn global_null() -> *const Value {
    Arc::into_raw(Arc::new(Value::Null))
}

pub unsafe extern "C" fn global_get(ctx: *mut Context, name: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! get {:?} !!", *name);

    let val = match *name {
        Value::Str(ref s) => {
            (*ctx).variables.get(s)
        }
        Value::Array(ref a) => {
            if let Value::Str(ref s) = *a[0] {
                (*ctx).variables.get(s)
            }
            else {
                None
            }
        }
        Value::Class(ref clss) => {
            let mut a = clss.lock().unwrap();
            if let Some(arr) = a.as_any().downcast_ref::<Array>() {
                if let Value::Str(ref s) = *arr.data[0] {
                    (*ctx).variables.get(s)
                }
                else {
                    None
                }
            }
            else {
                None
            }
        }
        _ => None,
    };

    if let Some(val) = val {
        return Arc::into_raw(val.clone());
    }

    0 as _
}

pub unsafe extern "C" fn get_func_addr(val: *const Value) -> u64 {
    #[cfg(feature = "trace")]
    log::trace!("!! get func {:?} !!", *val);

    match &*val {
        Value::Function(f) => {
            return *f;
        }
        Value::Method { class, func } => {
            return *func;
        }
        _ => {}
    }

    0
}

pub unsafe extern "C" fn global_set(
    ctx: *mut Context,
    name: *const Value,
    val: *mut Value,
) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! set {:?} = {:?} !!", *name, *val);

    match *name {
        Value::Str(ref s) => {
            (*ctx).variables.insert(
                s.clone(),
                Arc::from_raw(val),
            );
        }
        Value::Array(ref a) => {
            if let Value::Str(ref s) = *a[0] {
                (*ctx).variables.insert(
                    s.clone(),
                    Arc::from_raw(val),
                );
            }
        }
        Value::Class(ref clss) => {
            let mut a = clss.lock().unwrap();
            if let Some(arr) = a.as_any().downcast_ref::<Array>() {
                if let Value::Str(ref s) = *arr.data[0] {
                    (*ctx).variables.insert(
                        s.clone(),
                        Arc::from_raw(val),
                    );
                }
            }
        }
        _ => {}
    }

    0 as _
}

pub unsafe extern "C" fn get_attr(obj: *const Value, name: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! get-attr {:?} . {:?} !!", *obj, *name);

    let objv = Arc::from_raw(obj);
    let namev = Arc::from_raw(name);

    let ret = objv.get(namev.clone());

    Arc::into_raw(objv);
    Arc::into_raw(namev);
    Arc::into_raw(ret)
}

pub unsafe extern "C" fn to_bool(val: *const Value) -> i8 {
    let val = Arc::from_raw(val);
    let bool = val.to_bool();
    Arc::into_raw(val);
    bool as _
}

pub unsafe extern "C" fn add(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! add {:?} {:?} !!", *left, *right);

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Number(l + r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Number(0.0)))
}

pub unsafe extern "C" fn sub(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! sub !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Number(l - r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Number(0.0)))
}

pub unsafe extern "C" fn mul(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! mul !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Number(l * r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Number(0.0)))
}

pub unsafe extern "C" fn div(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! div !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Number(l / r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Number(0.0)))
}

pub unsafe extern "C" fn _mod(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! mod !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Number(l % r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Number(0.0)))
}

pub unsafe extern "C" fn gt(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! gt !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Boolean(l > r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Boolean(false)))
}

pub unsafe extern "C" fn gte(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! gte !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Boolean(l >= r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Boolean(false)))
}

pub unsafe extern "C" fn lt(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! lt !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Boolean(l < r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Boolean(false)))
}

pub unsafe extern "C" fn lte(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! lte !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Boolean(l <= r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Boolean(false)))
}

pub unsafe extern "C" fn eq(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! eq !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    Arc::into_raw(Arc::new(Value::Boolean(*left_rc == *right_rc)))
}

pub unsafe extern "C" fn neq(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! neq !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    Arc::into_raw(Arc::new(Value::Boolean(*left_rc != *right_rc)))
}

pub unsafe extern "C" fn and(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! and !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    Arc::into_raw(Arc::new(Value::Boolean(left_rc.to_bool() && right_rc.to_bool())))
}

pub unsafe extern "C" fn or(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! or !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    Arc::into_raw(Arc::new(Value::Boolean(left_rc.to_bool() || right_rc.to_bool())))
}


pub extern "C" fn string_new() -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! new string !!");
    Arc::into_raw(Arc::new(Value::Str("".to_owned())))
}

pub extern "C" fn string_copy(v: *const Value) -> *const Value {
    unsafe {
        #[cfg(feature = "trace")]
        log::trace!(target: "typescript.callback", "!! copy string {:?} !!", *v);
        Arc::into_raw( Arc::from_raw(v).clone() )
    }
}

pub unsafe extern "C" fn string_from(bytes: *mut i8) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! string from !!");
    // let data = CString::from_raw(bytes);
    let data = CStr::from_ptr(bytes);
    let owned = data.to_str().unwrap().to_string();
    Arc::into_raw(Arc::new(Value::Str(owned)))
}

pub extern "C" fn number_new(v: f64) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! new number {} !!", v);
    Arc::into_raw(Arc::new(Value::Number(v)))
}

pub unsafe extern "C" fn value_delete(a: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!("!! delete value {:?} !!", *a);

    Arc::from_raw(a);

    // &Value::Null as *const _
    0 as _
}