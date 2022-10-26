use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildCall2, LLVMBuildRet,
        LLVMFunctionType, LLVMPositionBuilderAtEnd,
    },
    prelude::LLVMValueRef,
};
use typescript_ast::ast;

use super::{statement::build_statements, Builder, InternFunction};

pub unsafe fn build_function(
    builder: &mut Builder,
    stmnt: &ast::function::Function,
) -> LLVMValueRef {
    let cname = CString::new(stmnt.name.clone().unwrap_or("generic".into())).unwrap();

    let mut args = Vec::new();
    for _ in 0..stmnt.params.len() {
        args.push(builder.p64t);
    }
    let func_t = LLVMFunctionType(builder.p64t, args.as_ptr() as _, args.len() as _, 0);
    let func = LLVMAddFunction(builder.module, cname.as_ptr(), func_t);

    if let Some(name) = stmnt.name.as_ref() {
        builder.function_cache.insert(
            name.clone(),
            InternFunction {
                func,
                ft: func_t,
                name: cname.clone(),
            },
        );
    }

    let bb = LLVMAppendBasicBlockInContext(builder.context, func, cname.as_ptr());

    let old_block = builder.current_block;
    builder.current_block = bb;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    let mut last = build_statements(builder, &stmnt.block);

    if last == 0 as _ {
        let null = builder.extern_functions.get("__global_null").unwrap();
        let args: Vec<LLVMValueRef> = Vec::new();
        last = LLVMBuildCall2(
            builder.builder,
            null.ft,
            null.func,
            args.as_ptr() as _,
            args.len() as u32,
            b"__null\0".as_ptr() as *const _,
        );
    }

    LLVMBuildRet(builder.builder, last);

    builder.current_block = old_block;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    func
}
