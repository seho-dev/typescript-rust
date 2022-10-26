use std::sync::Arc;

use llvm_sys::{core::{LLVMAppendBasicBlock, LLVMBuildBr, LLVMPositionBuilderAtEnd, LLVMBuildCall2, LLVMPointerType, LLVMBuildIntToPtr, LLVMConstInt, LLVMInt8TypeInContext, LLVMBuildICmp, LLVMBuildCondBr, LLVMFunctionType}, prelude::LLVMValueRef, LLVMIntPredicate};
use typescript_ast::ast::{statement::Statement, value::Value, repeat::Loop};

use super::{Builder, statement::build_statements, value::{build_value, build_get_attr, build_string, build_cmp, build_global_set}};



unsafe fn build_for(builder: &mut Builder, init: &Vec<Statement>, cond: &Arc<Value>, after: &Arc<Value>, block: &Vec<Statement>) {
    let for_loop = LLVMAppendBasicBlock(builder.current_function, b"for_init\0".as_ptr() as _);
    let for_cond = LLVMAppendBasicBlock(builder.current_function, b"for_cond\0".as_ptr() as _);
    let for_block = LLVMAppendBasicBlock(builder.current_function, b"for_block\0".as_ptr() as _);
    let for_after = LLVMAppendBasicBlock(builder.current_function, b"for_after\0".as_ptr() as _);
    let for_end = LLVMAppendBasicBlock(builder.current_function, b"for_end\0".as_ptr() as _);

    LLVMBuildBr(builder.builder, for_loop);

    builder.current_block = for_loop;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    build_statements(builder, init);

    LLVMBuildBr(builder.builder, for_cond);

    builder.current_block = for_cond;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    let cond = build_cmp(builder, cond.clone());
    let _if = LLVMBuildCondBr(builder.builder, cond, for_block, for_end);

    builder.current_block = for_block;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    build_statements(builder, block);

    LLVMBuildBr(builder.builder, for_after);

    builder.current_block = for_after;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    build_value(builder, after.clone());

    LLVMBuildBr(builder.builder, for_cond);

    builder.current_block = for_end;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);
}

unsafe fn build_for_of(builder: &mut Builder, name: &String, value: &Arc<Value>, block: &Vec<Statement>) {
    let for_loop = LLVMAppendBasicBlock(builder.current_function, b"for_of_init\0".as_ptr() as _);
    let for_cond = LLVMAppendBasicBlock(builder.current_function, b"for_of_cond\0".as_ptr() as _);
    let for_block = LLVMAppendBasicBlock(builder.current_function, b"for_of_block\0".as_ptr() as _);
    let for_end = LLVMAppendBasicBlock(builder.current_function, b"for_of_end\0".as_ptr() as _);

    LLVMBuildBr(builder.builder, for_loop);

    builder.current_block = for_loop;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    let name_ref = build_string(builder, name);
    let value_ref = build_value(builder, value.clone());
    let iter_ref = build_get_attr(builder, value_ref, "@iterator");
    let iter_addr = {
        let ex = builder.extern_functions.get("__get_func_addr").unwrap();
        let args = vec![iter_ref];

        LLVMBuildCall2(
            builder.builder, 
            ex.ft, 
            ex.func, 
            args.as_ptr() as _, 
            args.len() as _, 
            b"get_func_addr\0".as_ptr() as _
        )
    };
    let iter_ref = {
        let ret = builder.p64t;
        let params = vec![builder.p64t];
        let args: Vec<LLVMValueRef> = vec![value_ref];
        let ft = LLVMFunctionType(ret, params.as_ptr() as *mut _, params.len() as u32, 0);
        let ptr_type = LLVMPointerType(ft, 0);
        // let ptr_type = LLVMFunctionType(ctx.llvm_ptr, args.as_ptr() as *mut _, args.len() as u32, 0);
        let func_ptr = LLVMBuildIntToPtr(builder.builder, iter_addr, ptr_type, b"var_to_func\0".as_ptr() as *const _);
        LLVMBuildCall2(
            builder.builder, 
            ft, 
            func_ptr, 
            args.as_ptr() as _, 
            args.len() as _, 
            b"iter_get\0".as_ptr() as _
        )
    };

    let next_ref = build_get_attr(builder, iter_ref, "next");
    let next_addr = {
        let ex = builder.extern_functions.get("__get_func_addr").unwrap();
        let args = vec![next_ref];

        LLVMBuildCall2(
            builder.builder, 
            ex.ft, 
            ex.func, 
            args.as_ptr() as _, 
            args.len() as _, 
            b"get_func_addr\0".as_ptr() as _
        )
    };
    let next_ref = {
        let ret = builder.p64t;
        let args = vec![builder.p64t];
        let ft = LLVMFunctionType(ret, args.as_ptr() as *mut _, args.len() as u32, 0);
        let ptr_type = LLVMPointerType(ft, 0);
        // let ptr_type = LLVMFunctionType(ctx.llvm_ptr, args.as_ptr() as *mut _, args.len() as u32, 0);
        let func_ptr = LLVMBuildIntToPtr(builder.builder, next_addr, ptr_type, b"var_to_func\0".as_ptr() as *const _);
        func_ptr
        // LLVMBuildCall2(
        //     self.builder, 
        //     ft, 
        //     func_ptr, 
        //     args.as_ptr() as _, 
        //     0, 
        //     b"get_next\0".as_ptr() as _
        // )
    };

    LLVMBuildBr(builder.builder, for_cond);

    builder.current_block = for_cond;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    let step_ref = {
        let params = vec![builder.p64t];
        let args: Vec<LLVMValueRef> = vec![iter_ref];
        let ft = LLVMFunctionType(builder.p64t, params.as_ptr() as *mut _, params.len() as u32, 0);
        LLVMBuildCall2(
            builder.builder, 
            ft, 
            next_ref, 
            args.as_ptr() as _, 
            args.len() as _, 
            b"next\0".as_ptr() as _
        )
    };
    let done_ref = build_get_attr(builder, step_ref, "done");
    let cond = {
        let null = builder.extern_functions.get("__to_bool").unwrap();
        let args: Vec<LLVMValueRef> = vec![done_ref];
        LLVMBuildCall2(
            builder.builder,
            null.ft,
            null.func,
            args.as_ptr() as _,
            args.len() as u32,
            b"__to_bool\0".as_ptr() as *const _,
        )
    };

    let one = LLVMConstInt(LLVMInt8TypeInContext(builder.context), 1, 0);
    let cond = LLVMBuildICmp(
        builder.builder, 
        LLVMIntPredicate::LLVMIntEQ, 
        cond, 
        one, 
        b"cmp\0".as_ptr() as _
    );
    let _if = LLVMBuildCondBr(builder.builder, cond, for_block, for_end);

    builder.current_block = for_block;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);
    
    let value_ref = build_get_attr(builder, step_ref, "value");
    build_global_set(builder, name_ref, value_ref, false);

    build_statements(builder, block);

    LLVMBuildBr(builder.builder, for_cond);

    builder.current_block = for_end;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);
}


pub unsafe fn build_loop(builder: &mut Builder, stmnt: &Loop) {
    match stmnt {
        Loop::For{ init, cond, after, block } => {
            build_for(builder, init, cond, after, block);
        }
        Loop::ForOf { name, value, block } => {
            build_for_of(builder, name, value, block);
        }
        Loop::ForIn { name, value, block } => {}
        Loop::While {cond, block} => {}
    }
}