use llvm_sys::prelude::LLVMValueRef;
use typescript_ast::ast;

use super::{
    functions::build_function,
    ifs::build_if,
    repeat::build_loop,
    switch::build_switch,
    value::{build_string, build_value, build_global_set},
    Builder,
};

pub fn build_statement(
    builder: &mut Builder,
    statement: &ast::statement::Statement,
) -> LLVMValueRef {
    unsafe {
        match statement {
            ast::statement::Statement::Const { name, value } => {
                let name_ref = build_string(builder, name);
                let value_ref = build_value(builder, value.clone());
                build_global_set(builder, name_ref, value_ref, true)
            }
            ast::statement::Statement::Let { name, value } => {
                let name_ref = build_string(builder, name);
                let value_ref = build_value(builder, value.clone());
                build_global_set(builder, name_ref, value_ref, true)
            }
            ast::statement::Statement::Expression(call) => build_value(builder, call.clone()),
            ast::statement::Statement::Function(func) => build_function(builder, func),
            ast::statement::Statement::Return(val) => build_value(builder, val.clone()),
            ast::statement::Statement::If(ifelse) => build_if(builder, ifelse),
            ast::statement::Statement::Switch(switch) => build_switch(builder, switch),
            ast::statement::Statement::Loop(repeat) => {
                build_loop(builder, repeat);

                0 as _
            }
            _ => 0 as _,
        }
    }
}

pub fn build_statements(
    builder: &mut Builder,
    block: &Vec<ast::statement::Statement>,
) -> LLVMValueRef {
    let mut ret = 0 as _;

    for stmnt in block {
        ret = build_statement(builder, stmnt);
    }

    ret
}
