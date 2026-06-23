//! C code generator backend for Bunzo.
//!
//! This module translates an [`IrModule`] into standard C99 code
//! that can be compiled to a native machine binary.

use crate::diagnostics::CompilerError;
use crate::ir::{
    BinOpKind, Constant, Instruction, IrFunction, IrModule, IrType, Operand, UnaryOpKind,
    VirtualRegister,
};
use std::collections::HashMap;

/// Simple code writer with indentation helper.
struct CodeWriter {
    code: String,
    indent_level: usize,
}

impl CodeWriter {
    fn new() -> Self {
        Self {
            code: String::new(),
            indent_level: 0,
        }
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn outdent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn line(&mut self, s: impl AsRef<str>) {
        for _ in 0..self.indent_level {
            self.code.push_str("    ");
        }
        self.code.push_str(s.as_ref());
        self.code.push('\n');
    }

    fn finish(self) -> String {
        self.code
    }
}

/// Helper to infer and store types of virtual registers and local variables
/// in a function before generating code.
struct TypeInferrer<'a> {
    func: &'a IrFunction,
    module: &'a IrModule,
    global_vars: &'a HashMap<String, IrType>,
    reg_types: HashMap<VirtualRegister, IrType>,
    var_types: HashMap<String, IrType>,
}

impl<'a> TypeInferrer<'a> {
    fn new(
        func: &'a IrFunction,
        module: &'a IrModule,
        global_vars: &'a HashMap<String, IrType>,
    ) -> Self {
        let mut var_types = HashMap::new();
        // Initialize parameter types.
        for param in &func.params {
            var_types.insert(param.name.clone(), param.ty.clone());
        }
        Self {
            func,
            module,
            global_vars,
            reg_types: HashMap::new(),
            var_types,
        }
    }

    fn run(&mut self) {
        // Simple fixed-point iteration for type propagation.
        let mut changed = true;
        while changed {
            changed = false;

            for block in &self.func.blocks {
                for inst in &block.instructions {
                    match inst {
                        Instruction::Const { dest, ty: _, value } => {
                            let ty = self.type_of_constant(value);
                            if self.reg_types.insert(*dest, ty.clone()) != Some(ty) {
                                changed = true;
                            }
                        }
                        Instruction::Load { dest, name } => {
                            let ty = self
                                .var_types
                                .get(name)
                                .or_else(|| self.global_vars.get(name))
                                .cloned()
                                .unwrap_or(IrType::Any);
                            if self.reg_types.insert(*dest, ty.clone()) != Some(ty) {
                                changed = true;
                            }
                        }
                        Instruction::Store { name, value } => {
                            let ty = self.type_of_operand(value);
                            if ty != IrType::Void && ty != IrType::Null {
                                if self.var_types.insert(name.clone(), ty.clone()) != Some(ty) {
                                    changed = true;
                                }
                            }
                        }
                        Instruction::BinOp {
                            dest,
                            op,
                            left,
                            right: _,
                        } => {
                            let ty = match op {
                                BinOpKind::Equal
                                | BinOpKind::NotEqual
                                | BinOpKind::Less
                                | BinOpKind::Greater
                                | BinOpKind::LessEqual
                                | BinOpKind::GreaterEqual => IrType::Bool,
                                _ => self.type_of_operand(left),
                            };
                            if self.reg_types.insert(*dest, ty.clone()) != Some(ty) {
                                changed = true;
                            }
                        }
                        Instruction::UnaryOp { dest, op, operand } => {
                            let ty = match op {
                                UnaryOpKind::Not => IrType::Bool,
                                UnaryOpKind::Negate => self.type_of_operand(operand),
                            };
                            if self.reg_types.insert(*dest, ty.clone()) != Some(ty) {
                                changed = true;
                            }
                        }
                        Instruction::Call { dest, callee, .. } => {
                            let ty = self.type_of_operand(callee);
                            if self.reg_types.insert(*dest, ty.clone()) != Some(ty) {
                                changed = true;
                            }
                        }
                        Instruction::GetField { dest, .. } => {
                            if self.reg_types.insert(*dest, IrType::Null) != Some(IrType::Null) {
                                changed = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn type_of_constant(&self, c: &Constant) -> IrType {
        match c {
            Constant::Int(_) => IrType::Int,
            Constant::Float(_) => IrType::Float,
            Constant::String(_) => IrType::String,
            Constant::Bool(_) => IrType::Bool,
            Constant::Null => IrType::Null,
        }
    }

    fn type_of_operand(&self, op: &Operand) -> IrType {
        match op {
            Operand::Register(r) => self.reg_types.get(r).cloned().unwrap_or(IrType::Null),
            Operand::Constant(c) => self.type_of_constant(c),
            Operand::FunctionRef(name) => {
                if let Some(f) = self.module.get_function(name) {
                    f.return_type.clone()
                } else {
                    // Standard library functions.
                    if name.starts_with("math.") {
                        IrType::Float
                    } else if name == "print" {
                        IrType::Void
                    } else {
                        IrType::Any
                    }
                }
            }
        }
    }
}

/// Translates a Bunzo [`IrModule`] into a C source code string.
pub fn generate(module: &IrModule) -> Result<String, CompilerError> {
    let mut writer = CodeWriter::new();

    // 1. Scan and infer types for global variables (variables stored in __main__)
    let empty_globals = HashMap::new();
    let mut global_vars = HashMap::new();
    if let Some(main_func) = module.get_function("__main__") {
        let mut main_inferrer = TypeInferrer::new(main_func, module, &empty_globals);
        main_inferrer.run();
        global_vars = main_inferrer.var_types;
    }

    writer.line("/* Generated by Bunzo Compiler (bzc) */");
    writer.line("#include \"runtime.h\"");
    writer.line("#include <stdio.h>");
    writer.line("#include <stdlib.h>");
    writer.line("#include <stdbool.h>");
    writer.line("#include <stdint.h>");
    writer.line("#include <string.h>");
    writer.line("#include <math.h>");
    writer.line("");

    // 2. Global variable declarations
    writer.line("/* Global Variables */");
    let mut sorted_globals: Vec<_> = global_vars.keys().collect();
    sorted_globals.sort();
    for name in sorted_globals {
        let ty = global_vars.get(name).unwrap();
        let c_ty = ir_type_to_c_type(ty);
        writer.line(format!("{c_ty} var_{};", escape_c_name(name)));
    }
    writer.line("");

    // 3. Forward declarations
    writer.line("/* Forward Declarations */");
    for func in &module.functions {
        let ret_type = ir_type_to_c_type(&func.return_type);
        let name = escape_c_name(&func.name);
        let params_str = if func.params.is_empty() {
            "void".to_string()
        } else {
            func.params
                .iter()
                .map(|p| format!("{} {}", ir_type_to_c_type(&p.ty), escape_c_name(&p.name)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        writer.line(format!("{ret_type} {name}({params_str});"));
    }
    writer.line("");

    // 4. Translate functions
    for func in &module.functions {
        let ret_type = ir_type_to_c_type(&func.return_type);
        let name = escape_c_name(&func.name);
        let params_str = if func.params.is_empty() {
            "void".to_string()
        } else {
            func.params
                .iter()
                .map(|p| format!("{} {}", ir_type_to_c_type(&p.ty), escape_c_name(&p.name)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        writer.line(format!("{ret_type} {name}({params_str}) {{"));
        writer.indent();

        // Type Inference Pass for registers and locals.
        let mut inferrer = TypeInferrer::new(func, module, &global_vars);
        inferrer.run();

        // Declare virtual registers at the top.
        writer.line("/* Virtual Registers */");
        let mut sorted_regs: Vec<_> = inferrer.reg_types.keys().collect();
        sorted_regs.sort_by_key(|r| r.0);
        for reg in sorted_regs {
            let ty = inferrer.reg_types.get(reg).unwrap();
            let c_ty = ir_type_to_c_type(ty);
            writer.line(format!("{c_ty} _r{};", reg.0));
        }

        // Declare local variables at the top.
        writer.line("/* Local Variables */");
        let mut sorted_vars: Vec<_> = inferrer.var_types.keys().collect();
        sorted_vars.sort();
        for var in sorted_vars {
            // Exclude parameters and globals.
            if func.params.iter().any(|p| &p.name == var) || global_vars.contains_key(var) {
                continue;
            }
            let ty = inferrer.var_types.get(var).unwrap();
            let c_ty = ir_type_to_c_type(ty);
            writer.line(format!("{c_ty} var_{};", escape_c_name(var)));
        }
        writer.line("");

        // Generate Basic Blocks.
        for block in &func.blocks {
            writer.outdent();
            writer.line(format!("{}:", escape_c_block_label(&block.label)));
            writer.indent();

            for inst in &block.instructions {
                match inst {
                    Instruction::Const { dest, ty: _, value } => {
                        let c_val = format_constant(value);
                        writer.line(format!("_r{} = {};", dest.0, c_val));
                    }
                    Instruction::Load { dest, name } => {
                        // Check if it's a parameter, local variable, or global.
                        let is_param = func.params.iter().any(|p| &p.name == name);
                        let source_name = if is_param {
                            escape_c_name(name)
                        } else {
                            format!("var_{}", escape_c_name(name))
                        };
                        writer.line(format!("_r{} = {};", dest.0, source_name));
                    }
                    Instruction::Store { name, value } => {
                        let is_param = func.params.iter().any(|p| &p.name == name);
                        let dest_name = if is_param {
                            escape_c_name(name)
                        } else {
                            format!("var_{}", escape_c_name(name))
                        };
                        let val_str = format_operand(value, &inferrer);
                        writer.line(format!("{} = {};", dest_name, val_str));
                    }
                    Instruction::BinOp {
                        dest,
                        op,
                        left,
                        right,
                    } => {
                        let left_ty = inferrer.type_of_operand(left);
                        let right_ty = inferrer.type_of_operand(right);
                        let left_str = format_operand(left, &inferrer);
                        let right_str = format_operand(right, &inferrer);

                        if left_ty == IrType::String && right_ty == IrType::String {
                            // String operations.
                            match op {
                                BinOpKind::Add => {
                                    writer.line(format!(
                                        "_r{} = bunzo_concat_strings({}, {});",
                                        dest.0, left_str, right_str
                                    ));
                                }
                                BinOpKind::Equal => {
                                    writer.line(format!(
                                        "_r{} = bunzo_equal_strings({}, {});",
                                        dest.0, left_str, right_str
                                    ));
                                }
                                BinOpKind::NotEqual => {
                                    writer.line(format!(
                                        "_r{} = bunzo_notequal_strings({}, {});",
                                        dest.0, left_str, right_str
                                    ));
                                }
                                _ => {
                                    writer.line(format!(
                                        "_r{} = {} {} {};",
                                        dest.0,
                                        left_str,
                                        bin_op_to_c(op),
                                        right_str
                                    ));
                                }
                            }
                        } else if op == &BinOpKind::Modulo
                            && (left_ty == IrType::Float || right_ty == IrType::Float)
                        {
                            writer
                                .line(format!("_r{} = fmod({}, {});", dest.0, left_str, right_str));
                        } else {
                            writer.line(format!(
                                "_r{} = {} {} {};",
                                dest.0,
                                left_str,
                                bin_op_to_c(op),
                                right_str
                            ));
                        }
                    }
                    Instruction::UnaryOp { dest, op, operand } => {
                        let op_str = format_operand(operand, &inferrer);
                        let c_op = match op {
                            UnaryOpKind::Not => "!",
                            UnaryOpKind::Negate => "-",
                        };
                        writer.line(format!("_r{} = {}{};", dest.0, c_op, op_str));
                    }
                    Instruction::Print { value } => {
                        let ty = inferrer.type_of_operand(value);
                        let val_str = format_operand(value, &inferrer);
                        match ty {
                            IrType::Int => writer.line(format!("bunzo_print_int({});", val_str)),
                            IrType::Float => {
                                writer.line(format!("bunzo_print_float({});", val_str))
                            }
                            IrType::String => {
                                writer.line(format!("bunzo_print_string({});", val_str))
                            }
                            IrType::Bool => writer.line(format!("bunzo_print_bool({});", val_str)),
                            IrType::Null => writer.line("bunzo_print_null();"),
                            _ => writer.line(format!("bunzo_print_string({});", val_str)),
                        }
                    }
                    Instruction::Call { dest, callee, args } => {
                        let callee_str = format_operand(callee, &inferrer);
                        let args_str: Vec<_> =
                            args.iter().map(|a| format_operand(a, &inferrer)).collect();
                        writer.line(format!(
                            "_r{} = {}({});",
                            dest.0,
                            callee_str,
                            args_str.join(", ")
                        ));
                    }
                    Instruction::CallVoid { callee, args } => {
                        let callee_str = format_operand(callee, &inferrer);
                        let args_str: Vec<_> =
                            args.iter().map(|a| format_operand(a, &inferrer)).collect();
                        writer.line(format!("{}({});", callee_str, args_str.join(", ")));
                    }
                    Instruction::GetField { dest, .. } => {
                        writer.line(format!("_r{} = NULL;", dest.0));
                    }
                    Instruction::SetField { .. } => {}
                    Instruction::Jump { target } => {
                        writer.line(format!("goto {};", escape_c_block_label(target)));
                    }
                    Instruction::Branch {
                        condition,
                        then_label,
                        else_label,
                    } => {
                        let cond_str = format_operand(condition, &inferrer);
                        writer.line(format!(
                            "if ({}) goto {}; else goto {};",
                            cond_str,
                            escape_c_block_label(then_label),
                            escape_c_block_label(else_label)
                        ));
                    }
                    Instruction::Return { value } => {
                        if let Some(val) = value {
                            writer.line(format!("return {};", format_operand(val, &inferrer)));
                        } else {
                            writer.line("return;");
                        }
                    }
                }
            }
        }

        writer.outdent();
        writer.line("}");
        writer.line("");
    }

    // 5. Generate wrapper entry point (C main) calling __main__
    writer.line("/* Entry Point wrapper */");
    writer.line("int main(int argc, char** argv) {");
    writer.indent();
    writer.line("escape_main__main__();");
    writer.line("return 0;");
    writer.outdent();
    writer.line("}");

    Ok(writer.finish())
}

// ─── Formatting Helpers ───────────────────────────────────────────────────

fn ir_type_to_c_type(ty: &IrType) -> &'static str {
    match ty {
        IrType::Int => "int64_t",
        IrType::Float => "double",
        IrType::Bool => "bool",
        IrType::String => "const char*",
        IrType::Null => "void*",
        IrType::Void => "void",
        IrType::Class(_) => "void*",
        IrType::Struct(_) => "void*",
        IrType::Array(_) => "void*",
        IrType::Function { .. } => "void*",
        IrType::Any => "void*",
    }
}

fn escape_c_name(name: &str) -> String {
    let escaped = name.replace('.', "_");
    if escaped == "main" {
        "escape_main__main__".to_string()
    } else {
        escaped
    }
}

fn escape_c_block_label(label: &str) -> String {
    label.replace('.', "_")
}

fn format_operand(op: &Operand, _inferrer: &TypeInferrer) -> String {
    match op {
        Operand::Register(r) => format!("_r{}", r.0),
        Operand::Constant(c) => format_constant(c),
        Operand::FunctionRef(name) => escape_c_name(name),
    }
}

fn format_constant(c: &Constant) -> String {
    match c {
        Constant::Int(v) => format!("{}LL", v),
        Constant::Float(v) => {
            let s = format!("{:?}", v);
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                format!("{}.0", s)
            } else {
                s
            }
        }
        Constant::String(v) => format!("\"{}\"", escape_c_string(v)),
        Constant::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Constant::Null => "NULL".to_string(),
    }
}

fn escape_c_string(s: &str) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        match c {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn bin_op_to_c(op: &BinOpKind) -> &'static str {
    match op {
        BinOpKind::Add => "+",
        BinOpKind::Subtract => "-",
        BinOpKind::Multiply => "*",
        BinOpKind::Divide => "/",
        BinOpKind::Modulo => "%",
        BinOpKind::Equal => "==",
        BinOpKind::NotEqual => "!=",
        BinOpKind::Less => "<",
        BinOpKind::Greater => ">",
        BinOpKind::LessEqual => "<=",
        BinOpKind::GreaterEqual => ">=",
        BinOpKind::And => "&&",
        BinOpKind::Or => "||",
    }
}
