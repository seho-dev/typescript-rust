use llvm_sys::{
    core::{LLVMAppendBasicBlock, LLVMBuildBr, LLVMPositionBuilderAtEnd, LLVMBuildCondBr},
    prelude::LLVMValueRef,
};
use typescript_ast::ast::ifelse::IfElse;

use super::{Builder, value::build_cmp, statement::build_statements};

pub unsafe fn build_if(builder: &mut Builder, stmnt: &IfElse) -> LLVMValueRef {
    let ifblk = LLVMAppendBasicBlock(builder.current_function, b"if\0".as_ptr() as _);

    LLVMBuildBr(builder.builder, ifblk);

    builder.current_block = ifblk;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    let cond = build_cmp(builder, stmnt.expr.clone());

    let _then = LLVMAppendBasicBlock(builder.current_function, b"then\0".as_ptr() as _);
    let mut elseifs = Vec::new();
    // let mut incoming_value = Vec::new();
    // let mut incoming_block = Vec::new();

    for i in 0..stmnt.elseifs.len() {
        elseifs.push((
            LLVMAppendBasicBlock(builder.current_function, b"elseif_check\0".as_ptr() as _),
            LLVMAppendBasicBlock(builder.current_function, b"elseif\0".as_ptr() as _),
        ));
    }

    let _else = LLVMAppendBasicBlock(builder.current_function, b"else\0".as_ptr() as _);
    let _merge = LLVMAppendBasicBlock(builder.current_function, b"if_end\0".as_ptr() as _);

    let _if = LLVMBuildCondBr(
        builder.builder,
        cond,
        _then,
        if elseifs.len() > 1 {
            elseifs[0].0
        } else {
            _else
        },
    );

    builder.current_block = _then;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    let if_v = build_statements(builder, &stmnt.block);

    LLVMBuildBr(builder.builder, _merge);
    // let mut _then = LLVMGetInsertBlock(self.builder);
    // incoming_value.push(if_v);
    // incoming_block.push(_then);

    for i in 0..stmnt.elseifs.len() {
        let elseif = &stmnt.elseifs[i];
        let (check, body) = elseifs[i];
        let next_block = if i == stmnt.elseifs.len() - 1 {
            _else
        } else {
            elseifs[i + 1].0
        };
        builder.current_block = check;
        LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

        let cond = build_cmp(builder, elseif.expr.clone());
        let _if = LLVMBuildCondBr(builder.builder, cond, body, next_block);

        builder.current_block = body;
        LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

        let elseif_v = build_statements(builder, &elseif.block);

        LLVMBuildBr(builder.builder, _merge);
        // let mut _then = LLVMGetInsertBlock(self.builder);
        // incoming_value.push(elseif_v);
        // incoming_block.push(_then);
    }

    builder.current_block = _else;
    LLVMPositionBuilderAtEnd(builder.builder, builder.current_block);

    let else_v = build_statements(builder, &stmnt.els);

    LLVMBuildBr(builder.builder, _merge);
    // let mut _then = LLVMGetInsertBlock(self.builder);
    // incoming_value.push(else_v);
    // incoming_block.push(_then);

    builder.current_block = _merge;
    LLVMPositionBuilderAtEnd(builder.builder, _merge);

    // let phi = LLVMBuildPhi(self.builder, LLVMVoidType(), b"phi\0".as_ptr() as _);
    // LLVMAddIncoming(
    //     phi,
    //     incoming_value.as_mut_ptr(),
    //     incoming_block.as_mut_ptr(),
    //     incoming_block.len() as _
    // );

    // self.current_block = old_block;
    // LLVMPositionBuilderAtEnd(self.builder, self.current_block);

    // phi
    0 as _
}
