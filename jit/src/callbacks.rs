use std::{sync::Arc, ffi::CStr};

use super::{value::Value, context::Context};

pub unsafe extern "C" fn global_null() -> *const Value {
    Arc::into_raw(Arc::new(Value::Null))
}

pub unsafe extern "C" fn global_get(ctx: *mut Context, name: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! get {:?} !!", *name);

    if let Value::Array(ref a) = *name {
        if let Value::Str(ref s) = *a[0] {
            let val = (*ctx).variables.get(s);

            if let Some(val) = val {
                return Arc::into_raw(val.clone());
            }
        }
    }

    0 as _
}

pub unsafe extern "C" fn global_get_func(ctx: *mut Context, name: *const Value) -> usize {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! get func {:?} !!", *name);

    if let Value::Array(ref a) = *name {
        if let Value::Str(ref s) = *a[0] {
            let val = (*ctx).variables.get(s);

            // if let Value::Lambda(v) = **val.unwrap() {
            //     return v;
            // }
        }
    }

    0
}

pub unsafe extern "C" fn global_set(
    ctx: *mut Context,
    name: *const Value,
    val: *mut Value,
) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! set {:?} = {:?} !!", *name, *val);

    // if let Value::Array(ref a) = *name {
    //     log::trace!(target: "typescript.callback", "a = {:?}", a);

        if let Value::Str(ref s) = *name {
            (*ctx).variables.insert(
                s.clone(),
                Arc::from_raw(val),
            );
        }
    // }

    // &Value::Null as *const _
    0 as _
}


pub unsafe extern "C" fn add(left: *const Value, right: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! add !!");

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
    log::trace!(target: "typescript.callback", "!! sub !!");

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
    log::trace!(target: "typescript.callback", "!! mul !!");

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
    log::trace!(target: "typescript.callback", "!! div !!");

    let left_rc = Arc::from_raw(left);
    let right_rc = Arc::from_raw(right);

    if let Value::Number(l) = *left_rc {
        if let Value::Number(r) = *right_rc {
            return Arc::into_raw(Arc::new(Value::Number(l / r)));
        }
    }

    Arc::into_raw(Arc::new(Value::Number(0.0)))
}

pub extern "C" fn array_new() -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! new array !!");
    Arc::into_raw(Arc::new(Value::Array(Vec::new())))
}

pub unsafe extern "C" fn array_push(arr: *mut Value, v: *mut Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! pushing value !! {:?} {:?}", *arr, *v);

    if let Value::Array(ref mut a) = *arr {
        a.push(Arc::from_raw(v));
    }

    &Value::Null as *const _
}

pub extern "C" fn string_new() -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! new string !!");
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
    log::trace!(target: "typescript.callback", "!! string from !!");
    // let data = CString::from_raw(bytes);
    let data = CStr::from_ptr(bytes);
    let owned = data.to_str().unwrap().to_string();
    Arc::into_raw(Arc::new(Value::Str(owned)))
}

pub extern "C" fn number_new(v: f64) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! new number {} !!", v);
    Arc::into_raw(Arc::new(Value::Number(v)))
}

pub unsafe extern "C" fn value_delete(a: *const Value) -> *const Value {
    #[cfg(feature = "trace")]
    log::trace!(target: "typescript.callback", "!! delete value {:?} !!", *a);

    Arc::from_raw(a);

    // &Value::Null as *const _
    0 as _
}