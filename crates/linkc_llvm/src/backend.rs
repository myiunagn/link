use inkwell::prelude::*;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::types::{BasicType, BasicTypeEnum, FunctionType, PointerType};
use inkwell::values::*;
use inkwell::optimization::{pass_manager::PassManager, PassManagerBuilder};
use inkwell::targets::TargetMachine;
use linkc_parser::*;
use std::collections::HashMap;

pub struct LlvmBackend {
    context: Context,
    module: Module,
    builder: InkwellBuilder,
    var_map: HashMap<String, AllocatedValue>,
    fn_map: HashMap<String, FunctionValue>,
    tmp_counter: usize,
    has_main: bool,
    int_type: IntType,
    bool_type: IntType,
    void_type: VoidType,
    ptr_type: PointerType,
}

type InkwellBuilder = inkwell::builder::Builder<'static>;
type AllocatedValue = inkwell::values::PointerValue;

impl LlvmBackend {
    pub fn new() -> Self {
        let context = Context::create();
        let module = context.create_module("link_module");
        let builder = context.create_builder();

        let int_type = context.i64_type();
        let bool_type = context.bool_type();
        let void_type = context.void_type();
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

        Self {
            context,
            module,
            builder,
            var_map: HashMap::new(),
            fn_map: HashMap::new(),
            tmp_counter: 0,
            has_main: false,
            int_type,
            bool_type,
            void_type,
            ptr_type,
        }
    }

    fn fresh_tmp(&mut self) -> String {
        self.tmp_counter += 1;
        format!("_tmp_{}", self.tmp_counter)
    }

    fn map_type(&self, type_ann: &TypeAnnotation) -> BasicTypeEnum {
        match type_ann {
            TypeAnnotation::I8 | TypeAnnotation::I16 | TypeAnnotation::I32 | TypeAnnotation::I64 => {
                self.int_type.as_basic_type_enum()
            }
            TypeAnnotation::U8 | TypeAnnotation::U16 | TypeAnnotation::U32 | TypeAnnotation::U64 => {
                self.int_type.as_basic_type_enum()
            }
            TypeAnnotation::USize => self.int_type.as_basic_type_enum(),
            TypeAnnotation::F32 => self.context.f32_type().as_basic_type_enum(),
            TypeAnnotation::F64 => self.context.f64_type().as_basic_type_enum(),
            TypeAnnotation::Bool => self.bool_type.as_basic_type_enum(),
            TypeAnnotation::Str => self.ptr_type.as_basic_type_enum(),
            TypeAnnotation::Unit | TypeAnnotation::Void => self.void_type.as_basic_type_enum(),
            TypeAnnotation::Ptr(_) => self.ptr_type.as_basic_type_enum(),
            TypeAnnotation::Named(_) => self.ptr_type.as_basic_type_enum(),
            TypeAnnotation::Stream(_) => self.ptr_type.as_basic_type_enum(),
        }
    }

    fn default_type(&self) -> BasicTypeEnum {
        self.int_type.as_basic_type_enum()
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<inkwell::values::BasicValueEnum, String> {
        match expr {
            Expr::Int(n) => {
                Ok(self.int_type.const_int(*n as u64, true).as_basic_value_enum())
            }
            Expr::Float(f) => {
                Ok(self.context.f64_type().const_float(*f).as_basic_value_enum())
            }
            Expr::Bool(b) => {
                Ok(self.bool_type.const_int(if *b { 1 } else { 0 }, false).as_basic_value_enum())
            }
            Expr::None => {
                Ok(self.int_type.const_zero().as_basic_value_enum())
            }
            Expr::Ident(name) => {
                if let Some(alloc) = self.var_map.get(name) {
                    Ok(self.builder.build_load(self.int_type, *alloc, name)
                        .map_err(|e| format!("Load error: {}", e))?)
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            Expr::Binary { op, left, right } => {
                let left_val = self.generate_expr(left)?;
                let right_val = self.generate_expr(right)?;

                let (is_float, is_bool_comparison) = match (left_val, right_val) {
                    (BasicValueEnum::Float(_), BasicValueEnum::Float(_)) => (true, false),
                    (BasicValueEnum::Float(_), _) => (true, false),
                    (_, BasicValueEnum::Float(_)) => (true, false),
                    _ => (false, false),
                };

                if is_float {
                    self.generate_float_binop(op, left_val, right_val)
                } else if is_bool_comparison {
                    self.generate_int_binop(op, left_val, right_val)
                } else {
                    self.generate_int_binop(op, left_val, right_val)
                }
            }
            Expr::Unary { op, operand } => {
                let val = self.generate_expr(operand)?;
                match op {
                    UnaryOp::Neg => {
                        if let BasicValueEnum::Int(int_val) = val {
                            Ok(self.builder.build_neg(int_val, "neg")
                                .map_err(|e| format!("Neg error: {}", e))?
                                .as_basic_value_enum())
                        } else {
                            Err("Negation only supported for integer types".to_string())
                        }
                    }
                    UnaryOp::Not => {
                        if let BasicValueEnum::Bool(bool_val) = val {
                            Ok(self.builder.build_not(bool_val, "not")
                                .map_err(|e| format!("Not error: {}", e))?
                                .as_basic_value_enum())
                        } else {
                            Err("Logical not only supported for bool types".to_string())
                        }
                    }
                }
            }
            Expr::Call { callee, args } => {
                let fn_val = self.fn_map.get(callee)
                    .ok_or_else(|| format!("Undefined function: {}", callee))?;

                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.generate_expr(arg)?);
                }

                let call = self.builder.build_call(
                    *fn_val,
                    &arg_vals,
                    &format!("call_{}", callee),
                ).map_err(|e| format!("Call error: {}", e))?;

                if let Some(ret) = call {
                    Ok(ret)
                } else {
                    Ok(self.context.f64_type().const_float(0.0).as_basic_value_enum())
                }
            }
            Expr::IfExpr { condition, then_value, else_value } => {
                let cond = self.generate_expr(condition)?;
                let then_val = self.generate_expr(then_value)?;
                let else_val = self.generate_expr(else_value)?;
                Ok(self.builder.build_select(cond, then_val, else_val, "select")
                    .map_err(|e| format!("Select error: {}", e))?)
            }
            _ => Err("This expression type is not yet supported in LLVM backend".to_string()),
        }
    }

    fn generate_int_binop(
        &mut self,
        op: &BinOp,
        left: BasicValueEnum,
        right: BasicValueEnum,
    ) -> Result<BasicValueEnum, String> {
        let left_int = left.into_int();
        let right_int = right.into_int();

        match op {
            BinOp::Add => Ok(self.builder.build_int_add(left_int, right_int, "add")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Sub => Ok(self.builder.build_int_sub(left_int, right_int, "sub")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Mul => Ok(self.builder.build_int_mul(left_int, right_int, "mul")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Div => Ok(self.builder.build_int_signed_div(left_int, right_int, "sdiv")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Mod => Ok(self.builder.build_int_signed_rem(left_int, right_int, "srem")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Eq => Ok(self.builder.build_int_compare(
                inkwell::IntPredicate::EQ, left_int, right_int, "eq")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Neq => Ok(self.builder.build_int_compare(
                inkwell::IntPredicate::NE, left_int, right_int, "neq")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Lt => Ok(self.builder.build_int_compare(
                inkwell::IntPredicate::SLT, left_int, right_int, "slt")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Gt => Ok(self.builder.build_int_compare(
                inkwell::IntPredicate::SGT, left_int, right_int, "sgt")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::LtEq => Ok(self.builder.build_int_compare(
                inkwell::IntPredicate::SLE, left_int, right_int, "sle")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::GtEq => Ok(self.builder.build_int_compare(
                inkwell::IntPredicate::SGE, left_int, right_int, "sge")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::And => Ok(self.builder.build_and(left_int, right_int, "and")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Or => Ok(self.builder.build_or(left_int, right_int, "or")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Pipe => Err("pipe operator not supported in LLVM backend".to_string()),
        }
    }

    fn generate_float_binop(
        &mut self,
        op: &BinOp,
        left: BasicValueEnum,
        right: BasicValueEnum,
    ) -> Result<BasicValueEnum, String> {
        let left_float = left.into_float();
        let right_float = right.into_float();

        match op {
            BinOp::Add => Ok(self.builder.build_float_add(left_float, right_float, "fadd")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Sub => Ok(self.builder.build_float_sub(left_float, right_float, "fsub")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Mul => Ok(self.builder.build_float_mul(left_float, right_float, "fmul")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Div => Ok(self.builder.build_float_div(left_float, right_float, "fdiv")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Mod => Ok(self.builder.build_float_rem(left_float, right_float, "frem")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Eq => Ok(self.builder.build_float_compare(
                inkwell::FloatPredicate::OEQ, left_float, right_float, "foeq")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Neq => Ok(self.builder.build_float_compare(
                inkwell::FloatPredicate::ONE, left_float, right_float, "fone")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Lt => Ok(self.builder.build_float_compare(
                inkwell::FloatPredicate::OLT, left_float, right_float, "folt")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::Gt => Ok(self.builder.build_float_compare(
                inkwell::FloatPredicate::OGT, left_float, right_float, "fogt")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::LtEq => Ok(self.builder.build_float_compare(
                inkwell::FloatPredicate::OLE, left_float, right_float, "fole")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::GtEq => Ok(self.builder.build_float_compare(
                inkwell::FloatPredicate::OGE, left_float, right_float, "foge")
                .map_err(|e| e.to_string())?.as_basic_value_enum()),
            BinOp::And | BinOp::Or => Err("Logical operators not supported for floats".to_string()),
            BinOp::Pipe => Err("pipe operator not supported in LLVM backend".to_string()),
        }
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::LetDecl { name, type_annotation, value } => {
                let var_type = if let Some(ta) = type_annotation {
                    self.map_type(ta)
                } else {
                    self.default_type()
                };

                let alloc = self.builder.build_alloca(var_type, name)
                    .map_err(|e| format!("Alloca error: {}", e))?;

                if let Some(val_expr) = value {
                    let val = self.generate_expr(val_expr)?;
                    self.builder.build_store(alloc, val)
                        .map_err(|e| format!("Store error: {}", e))?;
                }

                self.var_map.insert(name.clone(), alloc);
            }
            Stmt::Assign { target, value } => {
                let val = self.generate_expr(value)?;
                let alloc = self.var_map.get(target)
                    .ok_or_else(|| format!("Undefined variable: {}", target))?;
                self.builder.build_store(*alloc, val)
                    .map_err(|e| format!("Store error: {}", e))?;
            }
            Stmt::Expr(expr) => {
                self.generate_expr(expr)?;
            }
            Stmt::Return(Some(expr)) => {
                let val = self.generate_expr(expr)?;
                self.builder.build_return(Some(&val))
                    .map_err(|e| format!("Return error: {}", e))?;
            }
            Stmt::Return(None) => {
                self.builder.build_return(None)
                    .map_err(|e| format!("Return error: {}", e))?;
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond = self.generate_expr(condition)?;

                let current_fn = self.builder.get_insert_block()
                    .and_then(|bb| bb.get_parent())
                    .ok_or_else(|| "No current function".to_string())?;

                let then_bb = self.context.append_basic_block(current_fn, "then");
                let else_bb = self.context.append_basic_block(current_fn, "else_block");
                let merge_bb = self.context.append_basic_block(current_fn, "merge");

                self.builder.build_conditional_branch(cond.as_int_value(), then_bb, else_bb)
                    .map_err(|e| format!("CondBr error: {}", e))?;

                self.builder.position_at_end(then_bb);
                self.generate_block(then_branch)?;
                if self.builder.get_insert_block().and_then(|bb| {
                    bb.get_terminator().map(|t| t.is_terminator())
                }).unwrap_or(false) {
                    // block already terminated
                } else {
                    self.builder.build_unconditional_branch(merge_bb)
                        .map_err(|e| e.to_string())?;
                }

                self.builder.position_at_end(else_bb);
                if let Some(else_block) = else_branch {
                    self.generate_block(else_block)?;
                }
                if self.builder.get_insert_block().and_then(|bb| {
                    bb.get_terminator().map(|t| t.is_terminator())
                }).unwrap_or(false) {
                    // block already terminated
                } else {
                    self.builder.build_unconditional_branch(merge_bb)
                        .map_err(|e| e.to_string())?;
                }

                self.builder.position_at_end(merge_bb);
            }
            Stmt::While { condition, body } => {
                let current_fn = self.builder.get_insert_block()
                    .and_then(|bb| bb.get_parent())
                    .ok_or_else(|| "No current function".to_string())?;

                let cond_bb = self.context.append_basic_block(current_fn, "while_cond");
                let body_bb = self.context.append_basic_block(current_fn, "while_body");
                let end_bb = self.context.append_basic_block(current_fn, "while_end");

                self.builder.build_unconditional_branch(cond_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(cond_bb);
                let cond = self.generate_expr(condition)?;
                self.builder.build_conditional_branch(cond.as_int_value(), body_bb, end_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(body_bb);
                self.generate_block(body)?;
                if self.builder.get_insert_block().and_then(|bb| {
                    bb.get_terminator().map(|t| t.is_terminator())
                }).unwrap_or(false) {
                    // already terminated
                } else {
                    self.builder.build_unconditional_branch(cond_bb)
                        .map_err(|e| e.to_string())?;
                }

                self.builder.position_at_end(end_bb);
            }
            Stmt::For { var_name, start, end, body } => {
                let current_fn = self.builder.get_insert_block()
                    .and_then(|bb| bb.get_parent())
                    .ok_or_else(|| "No current function".to_string())?;

                let start_val = self.generate_expr(start)?;
                let end_val = self.generate_expr(end)?;

                let alloc = self.builder.build_alloca(self.int_type, var_name)
                    .map_err(|e| format!("Alloca error: {}", e))?;
                self.builder.build_store(alloc, start_val)
                    .map_err(|e| e.to_string())?;

                self.var_map.insert(var_name.clone(), alloc);

                let cond_bb = self.context.append_basic_block(current_fn, "for_cond");
                let body_bb = self.context.append_basic_block(current_fn, "for_body");
                let end_bb = self.context.append_basic_block(current_fn, "for_end");

                self.builder.build_unconditional_branch(cond_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(cond_bb);
                let counter = self.builder.build_load(self.int_type, alloc, var_name)
                    .map_err(|e| e.to_string())?;
                let cmp = self.builder.build_int_compare(
                    inkwell::IntPredicate::SLT,
                    counter.into_int(),
                    end_val.into_int(),
                    "for_cmp",
                ).map_err(|e| e.to_string())?;
                self.builder.build_conditional_branch(cmp, body_bb, end_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(body_bb);
                self.generate_block(body)?;
                let current = self.builder.build_load(self.int_type, alloc, var_name)
                    .map_err(|e| e.to_string())?;
                let inc = self.builder.build_int_add(current.into_int(), self.int_type.const_int(1, true), "for_inc")
                    .map_err(|e| e.to_string())?;
                self.builder.build_store(alloc, inc.as_basic_value_enum())
                    .map_err(|e| e.to_string())?;
                self.builder.build_unconditional_branch(cond_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(end_bb);
            }
            Stmt::Loop(body) => {
                let current_fn = self.builder.get_insert_block()
                    .and_then(|bb| bb.get_parent())
                    .ok_or_else(|| "No current function".to_string())?;

                let body_bb = self.context.append_basic_block(current_fn, "loop_body");
                let end_bb = self.context.append_basic_block(current_fn, "loop_end");

                self.builder.build_unconditional_branch(body_bb)
                    .map_err(|e| e.to_string())?;

                self.builder.position_at_end(body_bb);
                self.generate_block(body)?;
                if self.builder.get_insert_block().and_then(|bb| {
                    bb.get_terminator().map(|t| t.is_terminator())
                }).unwrap_or(false) {
                    // already terminated
                } else {
                    self.builder.build_unconditional_branch(body_bb)
                        .map_err(|e| e.to_string())?;
                }

                self.builder.position_at_end(end_bb);
            }
            Stmt::Break => {
                self.builder.build_unconditional_branch(
                    self.context.append_basic_block(
                        self.builder.get_insert_block()
                            .and_then(|bb| bb.get_parent())
                            .ok_or_else(|| "No current function".to_string())?,
                        "break_target"
                    )
                ).map_err(|e| e.to_string())?;
            }
            Stmt::Continue => {
                self.builder.build_unconditional_branch(
                    self.context.append_basic_block(
                        self.builder.get_insert_block()
                            .and_then(|bb| bb.get_parent())
                            .ok_or_else(|| "No current function".to_string())?,
                        "continue_target"
                    )
                ).map_err(|e| e.to_string())?;
            }
            Stmt::FnDecl { name, params, return_type, body, .. } => {
                if name == "main" {
                    self.has_main = true;
                }

                let ret_type = if let Some(rt) = return_type {
                    self.map_type(rt)
                } else {
                    self.void_type.as_basic_type_enum()
                };

                let mut param_types = Vec::new();
                for (_, ptype) in params {
                    param_types.push(self.map_type(ptype));
                }

                let fn_type = self.context.ordered_types(&param_types, ret_type, false);
                let fn_val = self.module.add_function(name, fn_type, inkwell::module::Linkage::External);

                let entry_bb = self.context.append_basic_block(fn_val, "entry");
                self.builder.position_at_end(entry_bb);

                let mut fn_var_map = HashMap::new();
                for (i, (pname, _)) in params.iter().enumerate() {
                    if let Some(param) = fn_val.get_nth_param(i as u32) {
                        let alloc = self.builder.build_alloca(self.int_type, pname)
                            .map_err(|e| format!("Alloca error: {}", e))?;
                        self.builder.build_store(alloc, param)
                            .map_err(|e| e.to_string())?;
                        fn_var_map.insert(pname.clone(), alloc);
                    }
                }

                let saved_var_map = std::mem::replace(&mut self.var_map, fn_var_map);
                self.generate_block(body)?;

                if !self.builder.get_insert_block().and_then(|bb| {
                    bb.get_terminator().map(|t| t.is_terminator())
                }).unwrap_or(false) {
                    if ret_type == self.void_type.as_basic_type_enum() {
                        self.builder.build_return(None).map_err(|e| e.to_string())?;
                    } else {
                        let zero = self.int_type.const_zero();
                        self.builder.build_return(Some(&zero.as_basic_value_enum()))
                            .map_err(|e| e.to_string())?;
                    }
                }

                self.var_map = saved_var_map;
                self.fn_map.insert(name.clone(), fn_val);
            }
            _ => {
                return Err("This statement type is not yet supported in LLVM backend".to_string());
            }
        }

        Ok(())
    }

    fn generate_block(&mut self, block: &Block) -> Result<(), String> {
        for stmt in &block.stmts {
            self.generate_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, String> {
        let Program::Block(stmts) = program;

        let mut top_level_exprs = Vec::new();

        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    top_level_exprs.push(expr.clone());
                }
                Stmt::FnDecl { .. } => {
                    self.generate_stmt(stmt)?;
                }
                _ => {
                    self.generate_stmt(stmt)?;
                }
            }
        }

        if !top_level_exprs.is_empty() && !self.has_main {
            let ret_type = self.int_type.as_basic_type_enum();
            let fn_type = self.context.ordered_types(&[], ret_type, false);
            let main_fn = self.module.add_function("main", fn_type, inkwell::module::Linkage::External);

            let entry_bb = self.context.append_basic_block(main_fn, "entry");
            self.builder.position_at_end(entry_bb);

            for expr in &top_level_exprs {
                self.generate_expr(expr)?;
            }

            let zero = self.int_type.const_zero();
            self.builder.build_return(Some(&zero.as_basic_value_enum()))
                .map_err(|e| e.to_string())?;

            self.has_main = true;
        }

        Ok(self.module.print_to_string().to_string())
    }

    pub fn compile_to_ir(&mut self, program: &Program) -> Result<String, String> {
        self.generate(program)
    }

    pub fn compile_to_native(&mut self, program: &Program, output_path: &str) -> Result<String, String> {
        Target::initialize_x86(&InitializationConfig::default());

        self.generate(program)?;

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| format!("Target creation failed: {}", e))?;

        let target_machine = target.create_target_machine(
            &target_triple,
            "generic",
            "",
            inkwell::targets::CodeModel::Default,
        ).ok_or_else(|| "Failed to create target machine".to_string())?;

        let pm = PassManager::create();

        let pm_builder = PassManagerBuilder::create();
        pm_builder.set_optimization_level(inkwell::optimization::OptimizationLevel::Aggressive);
        pm_builder.populate_module_pass_manager(&pm);

        pm.run(&self.module, &target_machine)
            .map_err(|e| format!("Pass manager error: {}", e))?;

        let object_buf = target_machine.write_to_memory_buffer(&self.module, inkwell::targets::FileType::Object)
            .map_err(|e| format!("Object file generation failed: {}", e))?;

        let obj_path = format!("{}.o", output_path);
        std::fs::write(&obj_path, object_buf.as_slice())
            .map_err(|e| format!("Failed to write object file: {}", e))?;

        Ok(output_path.to_string())
    }
}
