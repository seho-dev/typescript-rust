use std::{
    collections::HashMap,
    error::Error,
    ffi::{c_void, CStr, CString},
    fs::File,
    io::Write,
    sync::Arc,
    time::SystemTime,
};

use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMAddGlobal, LLVMAppendBasicBlockInContext, LLVMBuildCall2,
        LLVMBuildFAdd, LLVMBuildFSub, LLVMBuildGlobalString, LLVMBuildGlobalStringPtr,
        LLVMBuildRet, LLVMBuildRetVoid, LLVMConstReal, LLVMContextCreate, LLVMContextDispose,
        LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMDoubleTypeInContext, LLVMFunctionType,
        LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMModuleCreateWithNameInContext,
        LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMPrintModuleToString, LLVMVoidTypeInContext, LLVMBuildCondBr, LLVMAppendBasicBlock, LLVMBuildBr, LLVMBuildPhi, LLVMBuildICmp, LLVMConstInt, LLVMAddIncoming, LLVMGetInsertBlock, LLVMDisposeMessage, LLVMVoidType,
    },
    execution_engine::{
        LLVMAddGlobalMapping, LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine,
        LLVMGetFunctionAddress, LLVMOpaqueExecutionEngine,
    },
    prelude::{
        LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef,
    }, LLVMBasicBlock, LLVMIntPredicate, analysis::{LLVMVerifyModule, LLVMVerifierFailureAction},
};

use typescript_ast::ast::{self, operation::AssignOperation};

use crate::error::JitError;

use super::{callbacks, context::Context, value::Value};

pub struct ExternFunction {
    pointer: *mut c_void,
    func: LLVMValueRef,
    ft: LLVMTypeRef,
    name: CString,
}

pub struct InternFunction {
    func: LLVMValueRef,
    ft: LLVMTypeRef,
    name: CString,
}

pub struct Module {
    id: Vec<u8>,
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    current_function: LLVMValueRef,
    current_block: LLVMBasicBlockRef,
    f64t: LLVMTypeRef,
    p64t: LLVMTypeRef,
    string_cache: HashMap<String, LLVMValueRef>,
    ee: *mut LLVMOpaqueExecutionEngine,
    extern_functions: HashMap<String, ExternFunction>,
    function_cache: HashMap<String, InternFunction>,
    pub namespace: Arc<Context>,
    namespace_ptr: LLVMValueRef,
}

extern "C" fn print(val: *const Value) {
    unsafe {
        println!("> {:?}", *val);
    }
}

impl Module {
    pub fn id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn from_ast(id: Vec<u8>, m: ast::Module, save_ir: Option<String>) -> Result<Module, Box<dyn Error>> {
        let id_hex = hex::encode(&id);
        let mut module = Self {
            id,
            builder: 0 as _,
            context: 0 as _,
            module: 0 as _,
            current_function: 0 as _,
            current_block: 0 as _,
            f64t: 0 as _,
            p64t: 0 as _,
            string_cache: HashMap::new(),
            ee: 0 as _,
            extern_functions: HashMap::new(),
            function_cache: HashMap::new(),
            namespace: Context::new(),
            namespace_ptr: 0 as _,
        };

        unsafe {
            module.context = LLVMContextCreate();
            module.module = LLVMModuleCreateWithNameInContext(
                id_hex.as_bytes().as_ptr() as *const _,
                module.context,
            );
            module.builder = LLVMCreateBuilderInContext(module.context);
            module.f64t = LLVMDoubleTypeInContext(module.context);
            module.p64t = LLVMPointerType(LLVMInt64TypeInContext(module.context), 0);

            module.namespace_ptr = LLVMAddGlobal(
                module.module,
                LLVMInt64TypeInContext(module.context),
                b"__context\0".as_ptr() as *const _,
            );

            module.add_fn("__global_null", callbacks::global_null as *mut _, 0);
            module.add_fn("__global_get", callbacks::global_get as *mut _, 2);
            // ctx.add_fn("__global_get_func", global_get_func as *mut _, 2);
            module.add_fn("__global_set", callbacks::global_set as *mut _, 3);
            module.add_fn("__value_delete", callbacks::value_delete as *mut _, 1);
            module.add_fn("__array_new", callbacks::array_new as *mut _, 0);
            module.add_fn("__array_push", callbacks::array_push as *mut _, 2);
            module.add_fn("__string_new", callbacks::string_new as *mut _, 0);
            module.add_fn("__string_copy", callbacks::string_copy as *mut _, 1);
            module.add_fn("__add", callbacks::add as *mut _, 2);
            module.add_fn("__sub", callbacks::sub as *mut _, 2);
            module.add_fn("__mul", callbacks::mul as *mut _, 2);
            module.add_fn("__div", callbacks::div as *mut _, 2);
            module.add_fn("__mod", callbacks::_mod as *mut _, 2);
            module.add_fn("__gt", callbacks::gt as *mut _, 2);
            module.add_fn("__gte", callbacks::gte as *mut _, 2);
            module.add_fn("__lt", callbacks::lt as *mut _, 2);
            module.add_fn("__lte", callbacks::lte as *mut _, 2);
            module.add_fn("__eq", callbacks::eq as *mut _, 2);
            module.add_fn("__neq", callbacks::neq as *mut _, 2);
            module.add_fn("__and", callbacks::and as *mut _, 2);
            module.add_fn("__or", callbacks::or as *mut _, 2);

            {
                let mut args = Vec::new();
                args.push(module.p64t);
                let ret = LLVMInt8TypeInContext(module.context);

                module.add_fn_with("__to_bool", ret, args, callbacks::to_bool as _);
            }
            {
                let mut args = Vec::new();
                args.push(LLVMPointerType(LLVMInt8TypeInContext(module.context), 0));

                module.add_fn_with("__string_from", module.p64t, args, callbacks::string_from as _);
            }
            {
                let mut args = Vec::new();
                args.push(module.f64t);

                module.add_fn_with("__number_new", module.p64t, args, callbacks::number_new as _);
            }
            module.add_fn("print", print as *mut _, 1);

            let main_func_t = LLVMFunctionType(
                LLVMVoidTypeInContext(module.context),
                std::ptr::null_mut(),
                0,
                0,
            );
            let main = LLVMAddFunction(
                module.module,
                b"__main__\0".as_ptr() as *const _,
                main_func_t,
            );
            module.current_function = main;

            let bb = LLVMAppendBasicBlockInContext(
                module.context,
                main,
                b"__main__entry\0".as_ptr() as *const _,
            );

            LLVMPositionBuilderAtEnd(module.builder, bb);
            module.current_block = bb;

            module.consume(&m);

            LLVMBuildRetVoid(module.builder);

            {
                let mut data = 0u8 as _;
                let ret = LLVMVerifyModule(module.module, LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut data);

                if ret != 0 {
                    let cast = CStr::from_ptr(data);
                    let error = JitError::ModuleVerify(cast.to_str().unwrap().into());
                    LLVMDisposeMessage(data);
                    LLVMDisposeBuilder(module.builder);
                    return Err(error.into())
                }
            }
            LLVMDisposeBuilder(module.builder);
            module.string_cache.clear();

            if let Some(ir) = save_ir {
                let data = LLVMPrintModuleToString(module.module);
                let cast = CStr::from_ptr(data);
                let mut dump = File::create(ir).unwrap();
                dump.write(cast.to_bytes()).unwrap();
            }

            module.ee = std::mem::zeroed();
            let mut out = std::mem::zeroed();

            LLVMCreateExecutionEngineForModule(&mut module.ee, module.module, &mut out);

            Ok(module)
        }
    }

    fn build_array(&self, parts: &Vec<LLVMValueRef>) -> LLVMValueRef {
        let an_ref = unsafe {
            let an = self.extern_functions.get("__array_new").unwrap();

            LLVMBuildCall2(
                self.builder,
                an.ft,
                an.func,
                0 as *mut LLVMValueRef,
                0,
                b"__array_new\0".as_ptr() as *const _,
            )
        };

        let array_push = self.extern_functions.get("__array_push").unwrap().clone();

        for p in parts {
            let args = vec![an_ref, *p];
            unsafe {
                LLVMBuildCall2(
                    self.builder,
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

    fn build_global_get(&self, name: LLVMValueRef, cleanup: bool) -> LLVMValueRef {
        let get_global = self.extern_functions.get("__global_get").unwrap();
        let value_delete = self.extern_functions.get("__value_delete").unwrap();
        let args = vec![self.namespace_ptr, name];
        let delete_args = vec![name];

        unsafe {
            let func = LLVMBuildCall2(
                self.builder,
                get_global.ft,
                get_global.func,
                args.as_ptr() as *mut _,
                args.len() as u32,
                b"global_get\0".as_ptr() as *const _,
            );

            if cleanup {
                LLVMBuildCall2(
                    self.builder,
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

    fn build_global_set(&self, name: LLVMValueRef, value: LLVMValueRef, cleanup: bool) -> LLVMValueRef {
        let global_set = self.extern_functions.get("__global_set").unwrap();
        let value_delete = self.extern_functions.get("__value_delete").unwrap();
        let args = vec![self.namespace_ptr, name, value];
        let delete_args = vec![name];

        unsafe {
            let ret = LLVMBuildCall2(
                self.builder,
                global_set.ft,
                global_set.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__global_set\0".as_ptr() as *const _,
            );

            if cleanup {
                LLVMBuildCall2(
                    self.builder,
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

    fn build_string(&mut self, s: &str) -> LLVMValueRef {
        unsafe {
            let cstr = if let Some(cstr) = self.string_cache.get(s) {
                *cstr
            } else {
                let cs = CString::new(s).unwrap();
                // let cstr = LLVMBuildGlobalString(
                //     self.builder,
                //     cs.as_ptr(),
                //     b"__str\0".as_ptr() as *const _,
                // );
                let cstr = LLVMBuildGlobalStringPtr(
                    self.builder,
                    cs.as_ptr(),
                    b"__str\0".as_ptr() as *const _,
                );

                self.string_cache.insert(s.to_string(), cstr);

                cstr
            };

            let args = vec![cstr];
            let string_from = self.extern_functions.get("__string_from").unwrap();
            LLVMBuildCall2(
                self.builder,
                string_from.ft,
                string_from.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__string_from\0".as_ptr() as *const _,
            )
        }
    }

    fn build_op(
        &self,
        op: &ast::operation::Operation,
        left: LLVMValueRef,
        right: LLVMValueRef,
    ) -> LLVMValueRef {
        unsafe {
            match op {
                ast::operation::Operation::Add => {
                    LLVMBuildFAdd(self.builder, left, right, b"__add\0".as_ptr() as _)
                }
                ast::operation::Operation::Sub => {
                    LLVMBuildFSub(self.builder, left, right, b"__sub\0".as_ptr() as _)
                }
                _ => 0 as _,
            }
        }
    }

    fn build_generic_op(
        &self,
        op: &ast::operation::Operation,
        left_ref: LLVMValueRef,
        right_ref: LLVMValueRef,
    ) -> LLVMValueRef {
        use ast::operation::Operation;

        let call = match op {
            Operation::Add => self.extern_functions.get("__add").unwrap(),
            Operation::Sub => self.extern_functions.get("__sub").unwrap(),
            Operation::Mul => self.extern_functions.get("__mul").unwrap(),
            Operation::Div => self.extern_functions.get("__div").unwrap(),
            Operation::Eq => self.extern_functions.get("__eq").unwrap(),
            Operation::Ne => self.extern_functions.get("__neq").unwrap(),
            Operation::Gt => self.extern_functions.get("__gt").unwrap(),
            Operation::Gte => self.extern_functions.get("__gte").unwrap(),
            Operation::Lt => self.extern_functions.get("__lt").unwrap(),
            Operation::Lte => self.extern_functions.get("__lte").unwrap(),
            Operation::Mod => self.extern_functions.get("__mod").unwrap(),
            Operation::And => self.extern_functions.get("__and").unwrap(),
            Operation::Or => self.extern_functions.get("__or").unwrap(),
        };

        let args = vec![left_ref, right_ref];
        unsafe {
            LLVMBuildCall2(
                self.builder,
                call.ft,
                call.func,
                args.as_ptr() as *mut LLVMValueRef,
                args.len() as u32,
                b"__op_res\0".as_ptr() as *const _,
            )
        }
    }

    fn build_value(&mut self, value: Arc<ast::value::Value>) -> LLVMValueRef {
        unsafe {
            match &*value {
                ast::value::Value::Number(n) => {
                    let float_new = self.extern_functions.get("__number_new").unwrap();
                    let args = vec![LLVMConstReal(self.f64t, *n)];
                    LLVMBuildCall2(
                        self.builder,
                        float_new.ft,
                        float_new.func,
                        args.as_ptr() as *mut LLVMValueRef,
                        args.len() as u32,
                        b"__number_new\0".as_ptr() as *const _,
                    )
                }
                ast::value::Value::String(n) => self.build_string(&n),
                ast::value::Value::Expression { left, op, right } => {
                    let left_ref = self.build_value(left.clone());
                    let right_ref = self.build_value(right.clone());
                    // self.build_op(op, left_ref, right_ref)
                    self.build_generic_op(op, left_ref, right_ref)
                }
                ast::value::Value::Identifier(n) => {
                    let parts = n.iter().map(|s| self.build_string(s)).collect();
                    let access = self.build_array(&parts);
                    self.build_global_get(access, true)
                }
                ast::value::Value::Array(a) => {
                    let mut values = Vec::new();

                    for p in a {
                        let p = self.build_value(p.clone());
                        values.push(p);
                    }

                    self.build_array(&values)
                }
                ast::value::Value::Call { identifier, args } => {
                    if identifier.len() == 1 {
                        if self.extern_functions.contains_key(&identifier[0]) {
                            let mut params: Vec<LLVMValueRef> = Vec::new();

                            for p in args {
                                params.push(self.build_value(p.clone()));
                            }

                            let n = self.extern_functions.get(&identifier[0]).unwrap();
                            return LLVMBuildCall2(
                                self.builder,
                                n.ft,
                                n.func,
                                params.as_ptr() as _,
                                params.len() as _,
                                b"__call_extern\0".as_ptr() as _,
                            );
                        } else if self.function_cache.contains_key(&identifier[0]) {
                            let mut params: Vec<LLVMValueRef> = Vec::new();

                            for p in args {
                                params.push(self.build_value(p.clone()));
                            }

                            let n = self.function_cache.get(&identifier[0]).unwrap();
                            return LLVMBuildCall2(
                                self.builder,
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
                ast::value::Value::Assign { identifier, op, value } => {
                    let name_ref = self.build_string(identifier);
                    let value_ref = self.build_value(value.clone());

                    if *op == AssignOperation::Neutral {
                        self.build_global_set(name_ref, value_ref, true)
                    }
                    else {
                        let old_ref = self.build_global_get(name_ref, false);
                        let new_ref = self.build_generic_op(&op.into(), old_ref, value_ref);
                        self.build_global_set(name_ref, new_ref, true)
                    }
                }
                ast::value::Value::Undefined => {
                    let null = self.extern_functions.get("__global_null").unwrap();
                    let args: Vec<LLVMValueRef> = Vec::new();
                    LLVMBuildCall2(
                        self.builder,
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
    }

    fn build_cmp(&mut self, val: Arc<ast::value::Value>) -> LLVMValueRef {
        unsafe {
            let cond = self.build_value(val.clone());
            
            let cond = {
                let null = self.extern_functions.get("__to_bool").unwrap();
                let args: Vec<LLVMValueRef> = vec![cond];
                LLVMBuildCall2(
                    self.builder,
                    null.ft,
                    null.func,
                    args.as_ptr() as *mut LLVMValueRef,
                    args.len() as u32,
                    b"__to_bool\0".as_ptr() as *const _,
                )
            };

            let one = LLVMConstInt(LLVMInt8TypeInContext(self.context), 1, 0);
            LLVMBuildICmp(
                self.builder, 
                LLVMIntPredicate::LLVMIntEQ, 
                cond, 
                one, 
                b"cmp\0".as_ptr() as _
            )
        }
    }

    fn build_if(&mut self, stmnt: &ast::ifelse::IfElse) -> LLVMValueRef {
        unsafe {
            let ifblk = LLVMAppendBasicBlock(self.current_function, b"if\0".as_ptr() as _);

            LLVMBuildBr(self.builder, ifblk);

            self.current_block = ifblk;
            LLVMPositionBuilderAtEnd(self.builder, self.current_block);

            let cond = self.build_cmp(stmnt.expr.clone());

            let _then = LLVMAppendBasicBlock(self.current_function, b"then\0".as_ptr() as _);
            let mut elseifs = Vec::new();
            // let mut incoming_value = Vec::new();
            // let mut incoming_block = Vec::new();

            for i in 0..stmnt.elseifs.len() {
                elseifs.push((
                    LLVMAppendBasicBlock(self.current_function, b"elseif_check\0".as_ptr() as _),
                    LLVMAppendBasicBlock(self.current_function, b"elseif\0".as_ptr() as _),
                ));
            }

            let _else = LLVMAppendBasicBlock(self.current_function, b"else\0".as_ptr() as _);
            let _merge = LLVMAppendBasicBlock(self.current_function, b"if_end\0".as_ptr() as _);

            let _if = LLVMBuildCondBr(self.builder, cond, _then, if elseifs.len() > 1 {elseifs[0].0} else { _else });

            self.current_block = _then;
            LLVMPositionBuilderAtEnd(self.builder, self.current_block);

            let if_v = self.consume_statements(&stmnt.block);

            LLVMBuildBr(self.builder, _merge);
            // let mut _then = LLVMGetInsertBlock(self.builder);
            // incoming_value.push(if_v);
            // incoming_block.push(_then);

            for i in 0..stmnt.elseifs.len() {
                let elseif = &stmnt.elseifs[i];
                let (check, body) = elseifs[i];
                let next_block = if i == stmnt.elseifs.len() - 1 {
                    _else
                }
                else {
                    elseifs[i + 1].0
                };
                self.current_block = check;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);

                let cond = self.build_cmp(elseif.expr.clone());
                let _if = LLVMBuildCondBr(self.builder, cond, body, next_block);

                self.current_block = body;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);

                let elseif_v = self.consume_statements(&elseif.block);

                LLVMBuildBr(self.builder, _merge);
                // let mut _then = LLVMGetInsertBlock(self.builder);
                // incoming_value.push(elseif_v);
                // incoming_block.push(_then);
            }

            self.current_block = _else;
            LLVMPositionBuilderAtEnd(self.builder, self.current_block);

            let else_v = self.consume_statements(&stmnt.els);

            LLVMBuildBr(self.builder, _merge);
            // let mut _then = LLVMGetInsertBlock(self.builder);
            // incoming_value.push(else_v);
            // incoming_block.push(_then);

            self.current_block = _merge;
            LLVMPositionBuilderAtEnd(self.builder, _merge);

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
    }

    fn build_switch(&mut self, switch: &ast::switch::Switch) -> LLVMValueRef {
        unsafe {
            let swblk = LLVMAppendBasicBlock(self.current_function, b"switch\0".as_ptr() as _);

            LLVMBuildBr(self.builder, swblk);

            self.current_block = swblk;
            LLVMPositionBuilderAtEnd(self.builder, self.current_block);

            let exp = self.build_value(switch.value.clone());

            let mut cases = Vec::new();

            for i in 0..switch.branches.len() {
                cases.push((
                    LLVMAppendBasicBlock(self.current_function, b"case_check\0".as_ptr() as _),
                    LLVMAppendBasicBlock(self.current_function, b"case\0".as_ptr() as _),
                ));
            }

            LLVMBuildBr(self.builder, cases[0].0);

            let default = LLVMAppendBasicBlock(self.current_function, b"default\0".as_ptr() as _);
            let _merge = LLVMAppendBasicBlock(self.current_function, b"switch_end\0".as_ptr() as _);

            for i in 0..switch.branches.len() {
                let case = &switch.branches[i];
                let (check, body) = cases[i];
                let next_block = if i == switch.branches.len() - 1 {
                    default
                }
                else {
                    cases[i + 1].0
                };
                self.current_block = check;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);

                let cond = self.build_value(case.expr.clone());
                let eq = {
                    let eq = self.extern_functions.get("__eq").unwrap();
                    let args: Vec<LLVMValueRef> = vec![exp, cond];
                    LLVMBuildCall2(
                        self.builder,
                        eq.ft,
                        eq.func,
                        args.as_ptr() as _,
                        args.len() as _,
                        b"__eq\0".as_ptr() as _,
                    )
                };
                let cond = {
                    let to_bool = self.extern_functions.get("__to_bool").unwrap();
                    let args: Vec<LLVMValueRef> = vec![eq];
                    LLVMBuildCall2(
                        self.builder,
                        to_bool.ft,
                        to_bool.func,
                        args.as_ptr() as _,
                        args.len() as _,
                        b"__to_bool\0".as_ptr() as _,
                    )
                };

                let one = LLVMConstInt(LLVMInt8TypeInContext(self.context), 1, 0);
                let cond = LLVMBuildICmp(
                    self.builder, 
                    LLVMIntPredicate::LLVMIntEQ, 
                    cond, 
                    one, 
                    b"cmp\0".as_ptr() as _
                );
                LLVMBuildCondBr(self.builder, cond, body, next_block);

                self.current_block = body;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);

                for stmnt in &case.block {
                    self.consume_statement(stmnt);
                }

                LLVMBuildBr(self.builder, _merge);
            }

            self.current_block = default;
            LLVMPositionBuilderAtEnd(self.builder, self.current_block);

            if let Some(default) = &switch.default {
                for stmnt in default {
                    self.consume_statement(stmnt);
                }
            }

            LLVMBuildBr(self.builder, _merge);
            // let mut _else = LLVMGetInsertBlock(self.builder);

            self.current_block = _merge;
            LLVMPositionBuilderAtEnd(self.builder, _merge);

            0 as _
        }
    }

    fn build_loop(&mut self, stmnt: &ast::repeat::Loop) -> LLVMValueRef {
        if let ast::repeat::Loop::For { init, cond, after, block } = stmnt {
            unsafe {
                let for_loop = LLVMAppendBasicBlock(self.current_function, b"for_init\0".as_ptr() as _);
                let for_cond = LLVMAppendBasicBlock(self.current_function, b"for_cond\0".as_ptr() as _);
                let for_block = LLVMAppendBasicBlock(self.current_function, b"for_block\0".as_ptr() as _);
                let for_after = LLVMAppendBasicBlock(self.current_function, b"for_after\0".as_ptr() as _);
                let for_end = LLVMAppendBasicBlock(self.current_function, b"for_end\0".as_ptr() as _);

                LLVMBuildBr(self.builder, for_loop);

                self.current_block = for_loop;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);

                self.consume_statements(init);

                LLVMBuildBr(self.builder, for_cond);

                self.current_block = for_cond;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);

                let cond = self.build_cmp(cond.clone());
                let _if = LLVMBuildCondBr(self.builder, cond, for_block, for_end);

                self.current_block = for_block;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);

                self.consume_statements(block);

                LLVMBuildBr(self.builder, for_after);

                self.current_block = for_after;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);

                self.build_value(after.clone());

                LLVMBuildBr(self.builder, for_cond);

                self.current_block = for_end;
                LLVMPositionBuilderAtEnd(self.builder, self.current_block);
            }
        }

        0 as _
    }

    fn build_function(&mut self, stmnt: &ast::function::Function) -> LLVMValueRef {
        unsafe {
            let cname = CString::new(stmnt.name.clone().unwrap_or("generic".into())).unwrap();

            let mut args = Vec::new();
            for _ in 0..stmnt.params.len() {
                args.push(self.p64t);
            }
            let func_t = LLVMFunctionType(self.p64t, args.as_ptr() as _, args.len() as _, 0);
            let func = LLVMAddFunction(self.module, cname.as_ptr(), func_t);

            if let Some(name) = stmnt.name.as_ref() {
                self.function_cache.insert(
                    name.clone(),
                    InternFunction {
                        func,
                        ft: func_t,
                        name: cname.clone(),
                    },
                );
            }

            let bb = LLVMAppendBasicBlockInContext(self.context, func, cname.as_ptr());

            let old_block = self.current_block;
            self.current_block = bb;
            LLVMPositionBuilderAtEnd(self.builder, self.current_block);

            let mut last = 0 as _;
            for stmnt in &stmnt.block {
                last = self.consume_statement(stmnt);
            }

            if last == 0 as _ {
                let null = self.extern_functions.get("__global_null").unwrap();
                let args: Vec<LLVMValueRef> = Vec::new();
                last = LLVMBuildCall2(
                    self.builder,
                    null.ft,
                    null.func,
                    args.as_ptr() as *mut LLVMValueRef,
                    args.len() as u32,
                    b"__null\0".as_ptr() as *const _,
                );
            }

            LLVMBuildRet(self.builder, last);

            self.current_block = old_block;
            LLVMPositionBuilderAtEnd(self.builder, self.current_block);

            func
        }
    }

    fn consume_statement(&mut self, statement: &ast::statement::Statement) -> LLVMValueRef {
        match statement {
            ast::statement::Statement::Const { name, value } => {
                let name_ref = self.build_string(name);
                let value_ref = self.build_value(value.clone());
                self.build_global_set(name_ref, value_ref, true)
            }
            ast::statement::Statement::Let { name, value } => {
                let name_ref = self.build_string(name);
                let value_ref = self.build_value(value.clone());
                self.build_global_set(name_ref, value_ref, true)
            }
            ast::statement::Statement::Expression(call) => self.build_value(call.clone()),
            ast::statement::Statement::Function(func) => self.build_function(func),
            ast::statement::Statement::Return(val) => self.build_value(val.clone()),
            ast::statement::Statement::If(ifelse) => self.build_if(ifelse),
            ast::statement::Statement::Switch(switch) => self.build_switch(switch),
            ast::statement::Statement::Loop(repeat) => self.build_loop(repeat),
            _ => 0 as _,
        }
    }

    fn consume_statements(&mut self, block: &Vec<ast::statement::Statement>) -> LLVMValueRef {
        let mut ret = 0 as _;

        for stmnt in block {
            ret = self.consume_statement(stmnt);
        }

        ret
    }

    fn consume(&mut self, module: &ast::Module) {
        for stmnt in &module.statements {
            self.consume_statement(stmnt);
        }
    }

    pub fn add_fn(&mut self, name: &str, f: *mut c_void, cnt: u32) {
        let mut args = Vec::new();
        for _ in 0..cnt {
            args.push(self.p64t);
        }

        self.add_fn_with(name, self.p64t, args, f);
    }

    pub fn add_fn_with(&mut self, name: &str, ret: LLVMTypeRef, args: Vec<LLVMTypeRef>, f: *mut c_void) {
        let cname = CString::new(name).unwrap();
        let (func, ft) = unsafe {
            let ft = LLVMFunctionType(ret, args.as_ptr() as *mut _, args.len() as u32, 0);
            (LLVMAddFunction(self.module, cname.as_ptr(), ft), ft)
        };

        let ex = ExternFunction {
            pointer: f,
            func,
            ft,
            name: cname,
        };
        self.extern_functions.insert(name.to_string(), ex);
    }

    pub fn run(&self) {
        unsafe {
            #[cfg(feature = "trace")]
            log::debug!("add gloabl mapping");

            let start = SystemTime::now();
            let ns_ptr = Arc::into_raw(self.namespace.clone());
            LLVMAddGlobalMapping(self.ee, self.namespace_ptr, ns_ptr as *mut _);

            for ex in self.extern_functions.values() {
                LLVMAddGlobalMapping(self.ee, ex.func, ex.pointer);

                // self.runtime_variables.insert(
                //     CString::new(name.as_bytes()).unwrap(),
                //     Rc::new(Value::Lambda(func as usize))
                // );
            }
            let dur = start.elapsed().unwrap();
            log::info!("create mapping: {}.{:06}", dur.as_secs(), dur.subsec_micros());

            let start = SystemTime::now();
            // JIT compiliation is defered until needed. So the real compilation starts here.
            let addr = LLVMGetFunctionAddress(self.ee, b"__main__\0".as_ptr() as *const _);
            let dur = start.elapsed().unwrap();
            log::info!("compilation: {}.{:06}", dur.as_secs(), dur.subsec_micros());

            let start = SystemTime::now();
            let f: extern "C" fn() = std::mem::transmute(addr);

            f();
            let dur = start.elapsed().unwrap();
            log::info!("main: {}.{:06}", dur.as_secs(), dur.subsec_micros());
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            if self.ee != 0 as _ {
                LLVMDisposeExecutionEngine(self.ee);
            }
            LLVMContextDispose(self.context);
        }
    }
}
