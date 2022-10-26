use std::{ffi::CString, sync::Arc};

use llvm_sys::{
    core::{LLVMBuildCall2, LLVMBuildGlobalStringPtr, LLVMConstReal, LLVMConstInt, LLVMInt8TypeInContext, LLVMBuildFAdd, LLVMBuildFSub, LLVMBuildICmp},
    prelude::LLVMValueRef, LLVMIntPredicate,
};
use typescript_ast::ast::{operation::{Operation, AssignOperation}, value::Value};

use super::Builder;

pub fn build_get_attr(builder: &mut Builder, obj: LLVMValueRef, name: &str) -> LLVMValueRef {
    let name_ref = build_string(builder, name);
    let ex = builder.extern_functions.get("__get_attr").unwrap();
    let args = vec![obj, name_ref];
    unsafe {
        LLVMBuildCall2(
            builder.builder,
            ex.ft,
            ex.func,
            args.as_ptr() as *mut LLVMValueRef,
            args.len() as u32,
            b"__get_attr\0".as_ptr() as *const _,
        )
    }
}

pub fn build_global_get(builder: &Builder, name: LLVMValueRef, cleanup: bool) -> LLVMValueRef {
    let get_global = builder.extern_functions.get("__global_get").unwrap();
    let value_delete = builder.extern_functions.get("__value_delete").unwrap();
    let args = vec![builder.namespace_ptr, name];
    let delete_args = vec![name];

    unsafe {
        let func = LLVMBuildCall2(
            builder.builder,
            get_global.ft,
            get_global.func,
            args.as_ptr() as *mut _,
            args.len() as u32,
            b"global_get\0".as_ptr() as *const _,
        );

        if cleanup {
            LLVMBuildCall2(
                builder.builder,
                value_delete.ft,
                value_delete.func,
                delete_args.as_ptr() as *mut LLVMValueRef,
                delete_args.len() as u32,
                b"__value_delete\0".as_ptr() as *const _,
            );
        }

        func
    }
}

pub fn build_global_set(builder: &Builder, name: LLVMValueRef, value: LLVMValueRef, cleanup: bool) -> LLVMValueRef {
    let global_set = builder.extern_functions.get("__global_set").unwrap();
    let value_delete = builder.extern_functions.get("__value_delete").unwrap();
    let args = vec![builder.namespace_ptr, name, value];
    let delete_args = vec![name];

    unsafe {
        let ret = LLVMBuildCall2(
            builder.builder,
            global_set.ft,
            global_set.func,
            args.as_ptr() as *mut LLVMValueRef,
            args.len() as u32,
            b"__global_set\0".as_ptr() as *const _,
        );

        if cleanup {
            LLVMBuildCall2(
                builder.builder,
                value_delete.ft,
                value_delete.func,
                delete_args.as_ptr() as *mut LLVMValueRef,
                delete_args.len() as u32,
                b"__value_delete\0".as_ptr() as *const _,
            );
        }

        ret
    }
}

pub fn build_cmp(builder: &mut Builder, val: Arc<Value>) -> LLVMValueRef {
    unsafe {
        let cond = build_value(builder, val.clone());

        let cond = {
            let null = builder.extern_functions.get("__to_bool").unwrap();
            let args: Vec<LLVMValueRef> = vec![cond];
            LLVMBuildCall2(
                builder.builder,
                null.ft,
                null.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__to_bool\0".as_ptr() as *const _,
            )
        };

        let one = LLVMConstInt(LLVMInt8TypeInContext(builder.context), 1, 0);
        LLVMBuildICmp(
            builder.builder,
            LLVMIntPredicate::LLVMIntEQ,
            cond,
            one,
            b"cmp\0".as_ptr() as _,
        )
    }
}

pub fn build_array(builder: &Builder, parts: &Vec<LLVMValueRef>) -> LLVMValueRef {
    let an_ref = unsafe {
        let an = builder.extern_functions.get("__array_new").unwrap();

        LLVMBuildCall2(
            builder.builder,
            an.ft,
            an.func,
            0 as *mut LLVMValueRef,
            0,
            b"__array_new\0".as_ptr() as *const _,
        )
    };

    let array_push = builder
        .extern_functions
        .get("__array_push")
        .unwrap()
        .clone();

    for p in parts {
        let args = vec![an_ref, *p];
        unsafe {
            LLVMBuildCall2(
                builder.builder,
                array_push.ft,
                array_push.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__array_push\0".as_ptr() as *const _,
            )
        };
    }

    an_ref
}

pub fn build_string(builder: &mut Builder, s: &str) -> LLVMValueRef {
    unsafe {
        let cstr = if let Some(cstr) = builder.string_cache.get(s) {
            *cstr
        } else {
            let cs = CString::new(s).unwrap();
            // let cstr = LLVMBuildGlobalString(
            //     self.builder,
            //     cs.as_ptr(),
            //     b"__str\0".as_ptr() as *const _,
            // );
            let cstr = LLVMBuildGlobalStringPtr(
                builder.builder,
                cs.as_ptr(),
                b"__str\0".as_ptr() as *const _,
            );

            builder.string_cache.insert(s.to_string(), cstr);

            cstr
        };

        let args = vec![cstr];
        let string_from = builder.extern_functions.get("__string_from").unwrap();
        LLVMBuildCall2(
            builder.builder,
            string_from.ft,
            string_from.func,
            args.as_ptr() as *mut LLVMValueRef,
            args.len() as u32,
            b"__string_from\0".as_ptr() as *const _,
        )
    }
}

fn build_op(
    builder: &mut Builder,
    op: &Operation,
    left: LLVMValueRef,
    right: LLVMValueRef,
) -> LLVMValueRef {
    unsafe {
        match op {
            Operation::Add => {
                LLVMBuildFAdd(builder.builder, left, right, b"__add\0".as_ptr() as _)
            }
            Operation::Sub => {
                LLVMBuildFSub(builder.builder, left, right, b"__sub\0".as_ptr() as _)
            }
            _ => 0 as _,
        }
    }
}

pub fn build_generic_op(
    builder: &Builder,
    op: &Operation,
    left_ref: LLVMValueRef,
    right_ref: LLVMValueRef,
) -> LLVMValueRef {

    let call = match op {
        Operation::Add => builder.extern_functions.get("__add").unwrap(),
        Operation::Sub => builder.extern_functions.get("__sub").unwrap(),
        Operation::Mul => builder.extern_functions.get("__mul").unwrap(),
        Operation::Div => builder.extern_functions.get("__div").unwrap(),
        Operation::Eq => builder.extern_functions.get("__eq").unwrap(),
        Operation::Ne => builder.extern_functions.get("__neq").unwrap(),
        Operation::Gt => builder.extern_functions.get("__gt").unwrap(),
        Operation::Gte => builder.extern_functions.get("__gte").unwrap(),
        Operation::Lt => builder.extern_functions.get("__lt").unwrap(),
        Operation::Lte => builder.extern_functions.get("__lte").unwrap(),
        Operation::Mod => builder.extern_functions.get("__mod").unwrap(),
        Operation::And => builder.extern_functions.get("__and").unwrap(),
        Operation::Or => builder.extern_functions.get("__or").unwrap(),
    };

    let args = vec![left_ref, right_ref];
    unsafe {
        LLVMBuildCall2(
            builder.builder,
            call.ft,
            call.func,
            args.as_ptr() as *mut LLVMValueRef,
            args.len() as u32,
            b"__op_res\0".as_ptr() as *const _,
        )
    }
}

pub unsafe fn build_value(builder: &mut Builder, value: Arc<Value>) -> LLVMValueRef {
    match &*value {
        Value::Number(n) => {
            let float_new = builder.extern_functions.get("__number_new").unwrap();
            let args = vec![LLVMConstReal(builder.f64t, *n)];
            LLVMBuildCall2(
                builder.builder,
                float_new.ft,
                float_new.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__number_new\0".as_ptr() as *const _,
            )
        }
        Value::String(n) => build_string(builder, &n),
        Value::Expression { left, op, right } => {
            let left_ref = build_value(builder, left.clone());
            let right_ref = build_value(builder, right.clone());
            // self.build_op(op, left_ref, right_ref)
            build_generic_op(builder, &op, left_ref, right_ref)
        }
        Value::Identifier(n) => {
            let parts = n.iter().map(|s| build_string(builder, s)).collect();
            let access = build_array(builder, &parts);
            build_global_get(builder, access, true)
        }
        Value::Array(a) => {
            let mut values = Vec::new();

            for p in a {
                let p = build_value(builder, p.clone());
                values.push(p);
            }

            build_array(builder, &values)
        }
        Value::Call { identifier, args } => {
            if identifier.len() == 1 {
                if builder.extern_functions.contains_key(&identifier[0]) {
                    let mut params: Vec<LLVMValueRef> = Vec::new();

                    for p in args {
                        params.push(build_value(builder, p.clone()));
                    }

                    let n = builder.extern_functions.get(&identifier[0]).unwrap();
                    return LLVMBuildCall2(
                        builder.builder,
                        n.ft,
                        n.func,
                        params.as_ptr() as _,
                        params.len() as _,
                        b"__call_extern\0".as_ptr() as _,
                    );
                } else if builder.function_cache.contains_key(&identifier[0]) {
                    let mut params: Vec<LLVMValueRef> = Vec::new();

                    for p in args {
                        params.push(build_value(builder, p.clone()));
                    }

                    let n = builder.function_cache.get(&identifier[0]).unwrap();
                    return LLVMBuildCall2(
                        builder.builder,
                        n.ft,
                        n.func,
                        params.as_ptr() as _,
                        params.len() as _,
                        b"__call_intern\0".as_ptr() as _,
                    );
                }
            }

            0 as _
        }
        Value::Assign {
            identifier,
            op,
            value,
        } => {
            let name_ref = build_string(builder, &*identifier);
            let value_ref = build_value(builder, value.clone());

            if *op == AssignOperation::Neutral {
                build_global_set(builder, name_ref, value_ref, true)
            } else {
                let old_ref = build_global_get(builder, name_ref, false);
                let new_ref = build_generic_op(builder, &op.into(), old_ref, value_ref);
                build_global_set(builder, name_ref, new_ref, true)
            }
        }
        Value::Undefined => {
            let null = builder.extern_functions.get("__global_null").unwrap();
            let args: Vec<LLVMValueRef> = Vec::new();
            LLVMBuildCall2(
                builder.builder,
                null.ft,
                null.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__null\0".as_ptr() as *const _,
            )
        }
        _ => {
            log::warn!("could not handle: {:?}", value);
            0 as _
        }
    }
}
