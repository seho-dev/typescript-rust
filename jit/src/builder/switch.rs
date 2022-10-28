use llvm_sys::{core::{LLVMAppendBasicBlock, LLVMBuildCall2, LLVMInt8TypeInContext, LLVMConstInt, LLVMBuildICmp, LLVMBuildBr, LLVMPositionBuilderAtEnd, LLVMBuildCondBr}, prelude::LLVMValueRef, LLVMIntPredicate};
use typescript_ast::ast::switch::Switch;

use super::{Builder, statement::build_statements, value::build_value};


pub unsafe fn build_switch(builder: &mut Builder, switch: &Switch) -> LLVMValueRef {
    let swblk = LLVMAppendBasicBlock(builder.current_function, b"switch\0".as_ptr() as _);

    LLVMBuildBr(builder.builder, swblk);

    builder.current_block = swblk;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    let exp = build_value(builder, switch.value.clone());

    let mut cases = Vec::new();

    for _ in 0..switch.branches.len() {
        cases.push((
            LLVMAppendBasicBlock(builder.current_function, b"case_check\0".as_ptr() as _),
            LLVMAppendBasicBlock(builder.current_function, b"case\0".as_ptr() as _),
        ));
    }

    LLVMBuildBr(builder.builder, cases[0].0);

    let default = LLVMAppendBasicBlock(builder.current_function, b"default\0".as_ptr() as _);
    let _merge = LLVMAppendBasicBlock(builder.current_function, b"switch_end\0".as_ptr() as _);

    for i in 0..switch.branches.len() {
        let case = &switch.branches[i];
        let (check, body) = cases[i];
        let next_block = if i == switch.branches.len() - 1 {
            default
        }
        else {
            cases[i + 1].0
        };
        builder.current_block = check;
        LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

        let cond = build_value(builder, case.expr.clone());
        let eq = {
            let eq = builder.extern_functions.get("__eq").unwrap();
            let args: Vec<LLVMValueRef> = vec![exp, cond];
            LLVMBuildCall2(
                builder.builder,
                eq.ft,
                eq.func,
                args.as_ptr() as _,
                args.len() as _,
                b"__eq\0".as_ptr() as _,
            )
        };
        let cond = {
            let to_bool = builder.extern_functions.get("__to_bool").unwrap();
            let args: Vec<LLVMValueRef> = vec![eq];
            LLVMBuildCall2(
                builder.builder,
                to_bool.ft,
                to_bool.func,
                args.as_ptr() as _,
                args.len() as _,
                b"__to_bool\0".as_ptr() as _,
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
        LLVMBuildCondBr(builder.builder, cond, body, next_block);

        builder.current_block = body;
        LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

        build_statements(builder, &case.block);

        LLVMBuildBr(builder.builder, _merge);
    }

    builder.current_block = default;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    if let Some(default) = &switch.default {
        build_statements(builder, default);
    }

    LLVMBuildBr(builder.builder, _merge);
    // let mut _else = LLVMGetInsertBlock(self.builder);

    builder.current_block = _merge;
    LLVMPositionBuilderAtEnd(builder.builder, _merge);

    0 as _
}