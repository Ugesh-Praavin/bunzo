//! Bunzo Static Type Checker.
//!
//! This module performs compile-time type checking of parsed programs,
//! ensuring that variable assignments, operations, function calls, and
//! struct/class usages are type-safe before execution.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{BinaryOperator, Expression, MatchPattern, Program, Statement, UnaryOperator};
use crate::diagnostics::CompilerError;

/// Representation of static types in Bunzo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Null,
    Array(Box<Type>),
    Struct(String),
    Class(String),
    Interface(String),
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Any,
    Void,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Null => write!(f, "null"),
            Type::Array(t) => write!(f, "Array<{}>", t),
            Type::Struct(name) => write!(f, "struct {}", name),
            Type::Class(name) => write!(f, "class {}", name),
            Type::Interface(name) => write!(f, "interface {}", name),
            Type::Function {
                params,
                return_type,
            } => {
                write!(f, "func(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Type::Any => write!(f, "any"),
            Type::Void => write!(f, "void"),
        }
    }
}

impl Type {
    /// Creates a Type from a type annotation name string.
    pub fn from_str(name: &str) -> Self {
        match name {
            "int" => Type::Int,
            "float" => Type::Float,
            "string" => Type::String,
            "bool" => Type::Bool,
            "null" => Type::Null,
            "void" => Type::Void,
            other => Type::Class(other.to_string()), // Default: resolve later
        }
    }
}

/// Lexical environment for type checking.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    parent: Option<Rc<RefCell<TypeEnv>>>,
    types: HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            parent: None,
            types: HashMap::new(),
        }
    }

    pub fn with_parent(parent: Rc<RefCell<TypeEnv>>) -> Self {
        TypeEnv {
            parent: Some(parent),
            types: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    pub fn lookup(&self, name: &str) -> Option<Type> {
        if let Some(ty) = self.types.get(name) {
            Some(ty.clone())
        } else if let Some(ref parent) = self.parent {
            parent.borrow().lookup(name)
        } else {
            None
        }
    }
}

/// The Type Checker driver.
pub struct TypeChecker {
    env: Rc<RefCell<TypeEnv>>,
    struct_defs: HashMap<String, HashMap<String, Type>>,
    class_fields: HashMap<String, HashMap<String, Type>>,
    class_methods: HashMap<String, HashMap<String, Type>>,
    class_extends: HashMap<String, Option<String>>,
    class_implements: HashMap<String, Vec<String>>,
    interface_methods: HashMap<String, HashMap<String, Type>>,
    expected_return_type: Option<Type>,
}

impl TypeChecker {
    /// Resolves generic/fallback types to their declared category.
    fn resolve_type(&self, ty: Type) -> Type {
        match ty {
            Type::Class(name) => {
                if self.struct_defs.contains_key(&name) {
                    Type::Struct(name)
                } else if self.interface_methods.contains_key(&name) {
                    Type::Interface(name)
                } else {
                    Type::Class(name)
                }
            }
            Type::Array(inner) => Type::Array(Box::new(self.resolve_type(*inner))),
            Type::Function {
                params,
                return_type,
            } => {
                let resolved_params = params.into_iter().map(|p| self.resolve_type(p)).collect();
                let resolved_ret = self.resolve_type(*return_type);
                Type::Function {
                    params: resolved_params,
                    return_type: Box::new(resolved_ret),
                }
            }
            other => other,
        }
    }

    fn resolve_type_name(&self, name: &str) -> Type {
        self.resolve_type(Type::from_str(name))
    }

    /// Checks if a source type is compatible with an expected target type.
    fn is_compatible(&self, found: &Type, expected: &Type) -> bool {
        if *found == Type::Any || *expected == Type::Any {
            return true;
        }
        if *found == Type::Null {
            return matches!(
                expected,
                Type::Null
                    | Type::Class(_)
                    | Type::Struct(_)
                    | Type::Interface(_)
                    | Type::Array(_)
                    | Type::Any
            );
        }
        if found == expected {
            return true;
        }
        if *found == Type::Int && *expected == Type::Float {
            return true;
        }
        if let (Type::Class(child), Type::Class(parent)) = (found, expected) {
            return self.is_subclass(child, parent);
        }
        if let (Type::Class(class_name), Type::Interface(iface_name)) = (found, expected) {
            return self.implements_interface(class_name, iface_name);
        }
        if let (Type::Array(a), Type::Array(b)) = (found, expected) {
            return self.is_compatible(a, b);
        }
        false
    }

    fn is_subclass(&self, child: &str, parent: &str) -> bool {
        let mut curr = Some(child.to_string());
        while let Some(curr_name) = curr {
            if curr_name == parent {
                return true;
            }
            curr = self.class_extends.get(&curr_name).and_then(|p| p.clone());
        }
        false
    }

    fn implements_interface(&self, class_name: &str, interface_name: &str) -> bool {
        let mut curr = Some(class_name.to_string());
        while let Some(curr_name) = curr {
            if let Some(ifaces) = self.class_implements.get(&curr_name) {
                if ifaces.contains(&interface_name.to_string()) {
                    return true;
                }
            }
            curr = self.class_extends.get(&curr_name).and_then(|p| p.clone());
        }
        false
    }

    /// Registers structures and types discovered globally.
    fn collect_declarations(&mut self, statements: &[Statement]) -> Result<(), CompilerError> {
        for stmt in statements {
            match stmt {
                Statement::StructDeclaration { name, fields, .. } => {
                    let mut field_map = HashMap::new();
                    for f in fields {
                        field_map.insert(f.name.clone(), Type::from_str(&f.type_name));
                    }
                    self.struct_defs.insert(name.clone(), field_map);
                }
                Statement::InterfaceDeclaration { name, methods, .. } => {
                    let mut method_map = HashMap::new();
                    for sig in methods {
                        let param_types: Vec<Type> = sig
                            .params
                            .iter()
                            .map(|p| Type::from_str(&p.type_name))
                            .collect();
                        let ret_type = sig
                            .return_type
                            .as_ref()
                            .map(|t| Type::from_str(t))
                            .unwrap_or(Type::Void);
                        method_map.insert(
                            sig.name.clone(),
                            Type::Function {
                                params: param_types,
                                return_type: Box::new(ret_type),
                            },
                        );
                    }
                    self.interface_methods.insert(name.clone(), method_map);
                }
                Statement::ClassDeclaration {
                    name,
                    extends,
                    implements,
                    fields,
                    methods,
                    ..
                } => {
                    self.class_extends.insert(name.clone(), extends.clone());
                    self.class_implements
                        .insert(name.clone(), implements.clone());

                    let mut field_map = HashMap::new();
                    for f in fields {
                        field_map.insert(f.name.clone(), Type::from_str(&f.type_name));
                    }
                    self.class_fields.insert(name.clone(), field_map);

                    let mut method_map = HashMap::new();
                    for m in methods {
                        if let Statement::FunctionDeclaration {
                            name: mname,
                            params,
                            return_type,
                            ..
                        } = m
                        {
                            let param_types: Vec<Type> = params
                                .iter()
                                .map(|p| Type::from_str(&p.type_name))
                                .collect();
                            let ret_type = return_type
                                .as_ref()
                                .map(|t| Type::from_str(t))
                                .unwrap_or(Type::Void);
                            method_map.insert(
                                mname.clone(),
                                Type::Function {
                                    params: param_types,
                                    return_type: Box::new(ret_type),
                                },
                            );
                        }
                    }
                    self.class_methods.insert(name.clone(), method_map);
                }
                Statement::FunctionDeclaration {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    let param_types: Vec<Type> = params
                        .iter()
                        .map(|p| Type::from_str(&p.type_name))
                        .collect();
                    let ret_type = return_type
                        .as_ref()
                        .map(|t| Type::from_str(t))
                        .unwrap_or(Type::Void);
                    let ty = Type::Function {
                        params: param_types,
                        return_type: Box::new(ret_type),
                    };
                    self.env.borrow_mut().define(name.clone(), ty);
                }
                _ => {}
            }
        }

        // Post-collect: resolve references
        let struct_defs_keys: Vec<String> = self.struct_defs.keys().cloned().collect();
        for sname in struct_defs_keys {
            let mut resolved_fields = Vec::new();
            if let Some(fields) = self.struct_defs.get(&sname) {
                for (fname, ty) in fields {
                    let resolved = self.resolve_type(ty.clone());
                    resolved_fields.push((fname.clone(), resolved));
                }
            }
            if let Some(fields) = self.struct_defs.get_mut(&sname) {
                for (fname, resolved) in resolved_fields {
                    fields.insert(fname, resolved);
                }
            }
        }

        let class_fields_keys: Vec<String> = self.class_fields.keys().cloned().collect();
        for cname in class_fields_keys {
            let mut resolved_fields = Vec::new();
            if let Some(fields) = self.class_fields.get(&cname) {
                for (fname, ty) in fields {
                    let resolved = self.resolve_type(ty.clone());
                    resolved_fields.push((fname.clone(), resolved));
                }
            }
            if let Some(fields) = self.class_fields.get_mut(&cname) {
                for (fname, resolved) in resolved_fields {
                    fields.insert(fname, resolved);
                }
            }
        }

        // Also define structures and classes in scope
        for sname in self.struct_defs.keys() {
            self.env
                .borrow_mut()
                .define(sname.clone(), Type::Struct(sname.clone()));
        }
        for cname in self.class_fields.keys() {
            self.env
                .borrow_mut()
                .define(cname.clone(), Type::Class(cname.clone()));
        }

        Ok(())
    }

    fn check_expr(&mut self, expr: &Expression) -> Result<Type, CompilerError> {
        match expr {
            Expression::IntegerLiteral { .. } => Ok(Type::Int),
            Expression::FloatLiteral { .. } => Ok(Type::Float),
            Expression::StringLiteral { .. } => Ok(Type::String),
            Expression::BooleanLiteral { .. } => Ok(Type::Bool),
            Expression::NullLiteral { .. } => Ok(Type::Null),
            Expression::Identifier { name, line, column } => {
                if name == "this" {
                    if let Some(ty) = self.env.borrow().lookup("this") {
                        return Ok(ty);
                    }
                }
                self.env
                    .borrow()
                    .lookup(name)
                    .ok_or_else(|| CompilerError::UndefinedVariable {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    })
            }
            Expression::Grouping { expression, .. } => self.check_expr(expression),
            Expression::UnaryOp {
                operator,
                operand,
                line,
                column,
            } => {
                let op_type = self.check_expr(operand)?;
                match operator {
                    UnaryOperator::LogicalNot => {
                        if !self.is_compatible(&op_type, &Type::Bool) {
                            return Err(CompilerError::TypeMismatch {
                                operation: "unary '!'".to_string(),
                                expected: "bool".to_string(),
                                found: op_type.to_string(),
                                line: *line,
                                column: *column,
                            });
                        }
                        Ok(Type::Bool)
                    }
                    UnaryOperator::Negate => {
                        if op_type != Type::Int && op_type != Type::Float && op_type != Type::Any {
                            return Err(CompilerError::TypeMismatch {
                                operation: "unary '-'".to_string(),
                                expected: "int or float".to_string(),
                                found: op_type.to_string(),
                                line: *line,
                                column: *column,
                            });
                        }
                        Ok(op_type)
                    }
                }
            }
            Expression::BinaryOp {
                operator,
                left,
                right,
                line,
                column,
            } => {
                let lt = self.check_expr(left)?;
                let rt = self.check_expr(right)?;
                match operator {
                    BinaryOperator::Add => {
                        if lt == Type::String || rt == Type::String {
                            if !self.is_compatible(&lt, &Type::String)
                                || !self.is_compatible(&rt, &Type::String)
                            {
                                return Err(CompilerError::TypeMismatch {
                                    operation: "string concatenation '+'".to_string(),
                                    expected: "string".to_string(),
                                    found: format!("{} and {}", lt, rt),
                                    line: *line,
                                    column: *column,
                                });
                            }
                            Ok(Type::String)
                        } else if lt == Type::Int && rt == Type::Int {
                            Ok(Type::Int)
                        } else if lt == Type::Float && rt == Type::Float {
                            Ok(Type::Float)
                        } else if (lt == Type::Int && rt == Type::Float)
                            || (lt == Type::Float && rt == Type::Int)
                        {
                            Ok(Type::Float)
                        } else if lt == Type::Any || rt == Type::Any {
                            Ok(Type::Any)
                        } else {
                            Err(CompilerError::TypeMismatch {
                                operation: "binary '+'".to_string(),
                                expected: "numeric or string types".to_string(),
                                found: format!("{} and {}", lt, rt),
                                line: *line,
                                column: *column,
                            })
                        }
                    }
                    BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide
                    | BinaryOperator::Modulo => {
                        if (lt == Type::Int || lt == Type::Float || lt == Type::Any)
                            && (rt == Type::Int || rt == Type::Float || rt == Type::Any)
                        {
                            if lt == Type::Float || rt == Type::Float {
                                Ok(Type::Float)
                            } else if lt == Type::Int && rt == Type::Int {
                                Ok(Type::Int)
                            } else {
                                Ok(Type::Any)
                            }
                        } else {
                            Err(CompilerError::TypeMismatch {
                                operation: format!("binary '{:?}'", operator),
                                expected: "int or float".to_string(),
                                found: format!("{} and {}", lt, rt),
                                line: *line,
                                column: *column,
                            })
                        }
                    }
                    BinaryOperator::Equal | BinaryOperator::NotEqual => Ok(Type::Bool),
                    BinaryOperator::Less
                    | BinaryOperator::Greater
                    | BinaryOperator::LessEqual
                    | BinaryOperator::GreaterEqual => {
                        if (lt == Type::Int
                            || lt == Type::Float
                            || lt == Type::String
                            || lt == Type::Any)
                            && (rt == Type::Int
                                || rt == Type::Float
                                || rt == Type::String
                                || rt == Type::Any)
                        {
                            Ok(Type::Bool)
                        } else {
                            Err(CompilerError::TypeMismatch {
                                operation: format!("comparison '{:?}'", operator),
                                expected: "int, float, or string".to_string(),
                                found: format!("{} and {}", lt, rt),
                                line: *line,
                                column: *column,
                            })
                        }
                    }
                    BinaryOperator::And | BinaryOperator::Or => {
                        if !self.is_compatible(&lt, &Type::Bool)
                            || !self.is_compatible(&rt, &Type::Bool)
                        {
                            return Err(CompilerError::TypeMismatch {
                                operation: format!("logical '{:?}'", operator),
                                expected: "bool".to_string(),
                                found: format!("{} and {}", lt, rt),
                                line: *line,
                                column: *column,
                            });
                        }
                        Ok(Type::Bool)
                    }
                }
            }
            Expression::Call {
                callee,
                arguments,
                line,
                column,
            } => {
                let callee_type = self.check_expr(callee)?;
                let mut arg_types = Vec::new();
                for arg in arguments {
                    arg_types.push(self.check_expr(arg)?);
                }

                match callee_type {
                    Type::Function {
                        params,
                        return_type,
                    } => {
                        if params.len() != arg_types.len() {
                            return Err(CompilerError::ArityMismatch {
                                name: "function call".to_string(),
                                expected: params.len(),
                                found: arg_types.len(),
                                line: *line,
                                column: *column,
                            });
                        }
                        for (i, (expected, found)) in
                            params.iter().zip(arg_types.iter()).enumerate()
                        {
                            if !self.is_compatible(found, expected) {
                                return Err(CompilerError::TypeMismatch {
                                    operation: format!("argument {} to function", i + 1),
                                    expected: expected.to_string(),
                                    found: found.to_string(),
                                    line: *line,
                                    column: *column,
                                });
                            }
                        }
                        Ok(*return_type)
                    }
                    Type::Class(name) => {
                        if let Some(methods) = self.class_methods.get(&name) {
                            if let Some(Type::Function { params, .. }) = methods.get("init") {
                                if params.len() != arg_types.len() {
                                    return Err(CompilerError::ArityMismatch {
                                        name: format!("{}.init", name),
                                        expected: params.len(),
                                        found: arg_types.len(),
                                        line: *line,
                                        column: *column,
                                    });
                                }
                                for (i, (expected, found)) in
                                    params.iter().zip(arg_types.iter()).enumerate()
                                {
                                    if !self.is_compatible(found, expected) {
                                        return Err(CompilerError::TypeMismatch {
                                            operation: format!(
                                                "argument {} to constructor of {}",
                                                i + 1,
                                                name
                                            ),
                                            expected: expected.to_string(),
                                            found: found.to_string(),
                                            line: *line,
                                            column: *column,
                                        });
                                    }
                                }
                            } else if !arg_types.is_empty() {
                                return Err(CompilerError::ArityMismatch {
                                    name: format!("{}.init", name),
                                    expected: 0,
                                    found: arg_types.len(),
                                    line: *line,
                                    column: *column,
                                });
                            }
                        }
                        Ok(Type::Class(name))
                    }
                    Type::Any => Ok(Type::Any),
                    other => Err(CompilerError::NotCallable {
                        found: other.to_string(),
                        line: *line,
                        column: *column,
                    }),
                }
            }
            Expression::StructLiteral {
                name,
                fields,
                line,
                column,
            } => {
                let declared_fields = self.struct_defs.get(name).cloned().ok_or_else(|| {
                    CompilerError::UnknownStruct {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    }
                })?;

                let mut initialized_fields = HashSet::new();
                for (fname, val_expr) in fields {
                    let expected_type =
                        declared_fields
                            .get(fname)
                            .ok_or_else(|| CompilerError::NoSuchField {
                                struct_name: name.clone(),
                                field: fname.clone(),
                                line: *line,
                                column: *column,
                            })?;
                    let val_type = self.check_expr(val_expr)?;
                    if !self.is_compatible(&val_type, expected_type) {
                        return Err(CompilerError::TypeMismatch {
                            operation: format!(
                                "field '{}' initialization in struct {}",
                                fname, name
                            ),
                            expected: expected_type.to_string(),
                            found: val_type.to_string(),
                            line: *line,
                            column: *column,
                        });
                    }
                    initialized_fields.insert(fname);
                }

                let mut missing = Vec::new();
                for fname in declared_fields.keys() {
                    if !initialized_fields.contains(fname) {
                        missing.push(fname.clone());
                    }
                }
                if !missing.is_empty() {
                    return Err(CompilerError::StructFieldMismatch {
                        struct_name: name.clone(),
                        missing,
                        unexpected: Vec::new(),
                        line: *line,
                        column: *column,
                    });
                }

                Ok(Type::Struct(name.clone()))
            }
            Expression::FieldAccess {
                object,
                field,
                line,
                column,
            } => {
                let obj_type = self.check_expr(object)?;
                match obj_type {
                    Type::Struct(ref sname) => {
                        let fields = self.struct_defs.get(sname).ok_or_else(|| {
                            CompilerError::UnknownStruct {
                                name: sname.clone(),
                                line: *line,
                                column: *column,
                            }
                        })?;
                        fields
                            .get(field)
                            .cloned()
                            .ok_or_else(|| CompilerError::NoSuchField {
                                struct_name: sname.clone(),
                                field: field.clone(),
                                line: *line,
                                column: *column,
                            })
                    }
                    Type::Class(ref cname) => {
                        if let Some(fields) = self.class_fields.get(cname) {
                            if let Some(ty) = fields.get(field) {
                                return Ok(ty.clone());
                            }
                        }
                        if let Some(methods) = self.class_methods.get(cname) {
                            if let Some(ty) = methods.get(field) {
                                return Ok(ty.clone());
                            }
                        }
                        let mut curr = self.class_extends.get(cname).and_then(|p| p.clone());
                        while let Some(parent_name) = curr {
                            if let Some(fields) = self.class_fields.get(&parent_name) {
                                if let Some(ty) = fields.get(field) {
                                    return Ok(ty.clone());
                                }
                            }
                            if let Some(methods) = self.class_methods.get(&parent_name) {
                                if let Some(ty) = methods.get(field) {
                                    return Ok(ty.clone());
                                }
                            }
                            curr = self.class_extends.get(&parent_name).and_then(|p| p.clone());
                        }

                        Err(CompilerError::NoSuchField {
                            struct_name: cname.clone(),
                            field: field.clone(),
                            line: *line,
                            column: *column,
                        })
                    }
                    Type::Any => Ok(Type::Any),
                    other => Err(CompilerError::TypeMismatch {
                        operation: "field access".to_string(),
                        expected: "Struct or Class".to_string(),
                        found: other.to_string(),
                        line: *line,
                        column: *column,
                    }),
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                if elements.is_empty() {
                    Ok(Type::Array(Box::new(Type::Any)))
                } else {
                    let mut element_type = self.check_expr(&elements[0])?;
                    for el in elements.iter().skip(1) {
                        let t = self.check_expr(el)?;
                        if t != element_type {
                            element_type = Type::Any;
                            break;
                        }
                    }
                    Ok(Type::Array(Box::new(element_type)))
                }
            }
            Expression::IndexExpression {
                object,
                index,
                line,
                column,
            } => {
                let obj_type = self.check_expr(object)?;
                let idx_type = self.check_expr(index)?;

                match obj_type {
                    Type::Array(inner) => {
                        if !self.is_compatible(&idx_type, &Type::Int) {
                            return Err(CompilerError::TypeMismatch {
                                operation: "array index".to_string(),
                                expected: "int".to_string(),
                                found: idx_type.to_string(),
                                line: *line,
                                column: *column,
                            });
                        }
                        Ok(*inner)
                    }
                    Type::Any => Ok(Type::Any),
                    other => Err(CompilerError::TypeMismatch {
                        operation: "indexing".to_string(),
                        expected: "array".to_string(),
                        found: other.to_string(),
                        line: *line,
                        column: *column,
                    }),
                }
            }
            Expression::EnumVariantExpr {
                enum_name, payload, ..
            } => {
                if let Some(p) = payload {
                    self.check_expr(p)?;
                }
                Ok(Type::Class(enum_name.clone()))
            }
            Expression::PropagateError { expression, .. } => self.check_expr(expression),
            Expression::MoveExpr { name, line, column } => self
                .env
                .borrow()
                .lookup(name)
                .ok_or_else(|| CompilerError::UndefinedVariable {
                    name: name.clone(),
                    line: *line,
                    column: *column,
                }),
            Expression::AwaitExpr { expression, .. } => self.check_expr(expression),
            Expression::SuperExpr { line, column } => {
                if let Some(this_class) = self.env.borrow().lookup("this") {
                    if let Type::Class(ref cname) = this_class {
                        if let Some(parent) = self.class_extends.get(cname).and_then(|p| p.clone())
                        {
                            return Ok(Type::Class(parent));
                        }
                    }
                }
                Err(CompilerError::UndefinedVariable {
                    name: "super".to_string(),
                    line: *line,
                    column: *column,
                })
            }
        }
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), CompilerError> {
        match stmt {
            Statement::LetDeclaration {
                name, initializer, ..
            } => {
                let ty = self.check_expr(initializer)?;
                self.env.borrow_mut().define(name.clone(), ty);
                Ok(())
            }
            Statement::ConstDeclaration {
                name, initializer, ..
            } => {
                let ty = self.check_expr(initializer)?;
                self.env.borrow_mut().define(name.clone(), ty);
                Ok(())
            }
            Statement::PrintStatement { argument, .. } => {
                self.check_expr(argument)?;
                Ok(())
            }
            Statement::ExpressionStatement { expression } => {
                self.check_expr(expression)?;
                Ok(())
            }
            Statement::FunctionDeclaration {
                name,
                params,
                return_type,
                body,
                ..
            } => {
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| self.resolve_type_name(&p.type_name))
                    .collect();
                let ret_type = return_type
                    .as_ref()
                    .map(|t| self.resolve_type_name(t))
                    .unwrap_or(Type::Void);

                let fn_type = Type::Function {
                    params: param_types.clone(),
                    return_type: Box::new(ret_type.clone()),
                };
                self.env.borrow_mut().define(name.clone(), fn_type);

                let parent_env = self.env.clone();
                self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));

                for (p, p_type) in params.iter().zip(param_types.iter()) {
                    self.env.borrow_mut().define(p.name.clone(), p_type.clone());
                }

                let old_return = self.expected_return_type.replace(ret_type);
                self.check_block(body)?;
                self.expected_return_type = old_return;

                self.env = parent_env;
                Ok(())
            }
            Statement::ReturnStatement {
                value,
                line,
                column,
            } => {
                let ret_type = if let Some(val) = value {
                    self.check_expr(val)?
                } else {
                    Type::Void
                };

                if let Some(ref expected) = self.expected_return_type {
                    if !self.is_compatible(&ret_type, expected) {
                        return Err(CompilerError::TypeMismatch {
                            operation: "return statement".to_string(),
                            expected: expected.to_string(),
                            found: ret_type.to_string(),
                            line: *line,
                            column: *column,
                        });
                    }
                }
                Ok(())
            }
            Statement::Assignment {
                name,
                value,
                line,
                column,
            } => {
                let val_type = self.check_expr(value)?;
                let var_type = self.env.borrow().lookup(name).ok_or_else(|| {
                    CompilerError::UndefinedVariable {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    }
                })?;

                if !self.is_compatible(&val_type, &var_type) {
                    return Err(CompilerError::TypeMismatch {
                        operation: format!("assignment to variable '{}'", name),
                        expected: var_type.to_string(),
                        found: val_type.to_string(),
                        line: *line,
                        column: *column,
                    });
                }
                Ok(())
            }
            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                line,
                column,
            } => {
                let cond_type = self.check_expr(condition)?;
                if !self.is_compatible(&cond_type, &Type::Bool) {
                    return Err(CompilerError::TypeMismatch {
                        operation: "if condition".to_string(),
                        expected: "bool".to_string(),
                        found: cond_type.to_string(),
                        line: *line,
                        column: *column,
                    });
                }

                let parent_env = self.env.clone();
                self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));
                self.check_block(then_branch)?;
                self.env = parent_env.clone();

                if let Some(els) = else_branch {
                    self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));
                    self.check_block(els)?;
                    self.env = parent_env;
                }
                Ok(())
            }
            Statement::WhileStatement {
                condition,
                body,
                line,
                column,
            } => {
                let cond_type = self.check_expr(condition)?;
                if !self.is_compatible(&cond_type, &Type::Bool) {
                    return Err(CompilerError::TypeMismatch {
                        operation: "while condition".to_string(),
                        expected: "bool".to_string(),
                        found: cond_type.to_string(),
                        line: *line,
                        column: *column,
                    });
                }
                let parent_env = self.env.clone();
                self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));
                self.check_block(body)?;
                self.env = parent_env;
                Ok(())
            }
            Statement::ForStatement {
                variable,
                start,
                end,
                body,
                line,
                column,
            } => {
                let st = self.check_expr(start)?;
                let et = self.check_expr(end)?;
                if !self.is_compatible(&st, &Type::Int) {
                    return Err(CompilerError::TypeMismatch {
                        operation: "for-loop range start".to_string(),
                        expected: "int".to_string(),
                        found: st.to_string(),
                        line: *line,
                        column: *column,
                    });
                }
                if !self.is_compatible(&et, &Type::Int) {
                    return Err(CompilerError::TypeMismatch {
                        operation: "for-loop range end".to_string(),
                        expected: "int".to_string(),
                        found: et.to_string(),
                        line: *line,
                        column: *column,
                    });
                }

                let parent_env = self.env.clone();
                self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));
                self.env.borrow_mut().define(variable.clone(), Type::Int);
                self.check_block(body)?;
                self.env = parent_env;
                Ok(())
            }
            Statement::BreakStatement { .. } | Statement::ContinueStatement { .. } => Ok(()),
            Statement::StructDeclaration { .. } => Ok(()),
            Statement::ClassDeclaration { name, methods, .. } => {
                let parent_env = self.env.clone();

                for m in methods {
                    if let Statement::FunctionDeclaration {
                        params,
                        return_type,
                        body,
                        ..
                    } = m
                    {
                        self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));
                        self.env
                            .borrow_mut()
                            .define("this".to_string(), Type::Class(name.clone()));

                        let param_types: Vec<Type> = params
                            .iter()
                            .map(|p| self.resolve_type_name(&p.type_name))
                            .collect();
                        let ret_type = return_type
                            .as_ref()
                            .map(|t| self.resolve_type_name(t))
                            .unwrap_or(Type::Void);

                        for (p, p_type) in params.iter().zip(param_types.iter()) {
                            self.env.borrow_mut().define(p.name.clone(), p_type.clone());
                        }

                        let old_return = self.expected_return_type.replace(ret_type);
                        self.check_block(body)?;
                        self.expected_return_type = old_return;
                    }
                }

                self.env = parent_env;
                Ok(())
            }
            Statement::FieldAssignment {
                object,
                field,
                value,
                line,
                column,
            } => {
                let obj_type = self.check_expr(object)?;
                let val_type = self.check_expr(value)?;

                match obj_type {
                    Type::Struct(ref sname) => {
                        let fields = self.struct_defs.get(sname).ok_or_else(|| {
                            CompilerError::UnknownStruct {
                                name: sname.clone(),
                                line: *line,
                                column: *column,
                            }
                        })?;
                        let expected_type =
                            fields
                                .get(field)
                                .ok_or_else(|| CompilerError::NoSuchField {
                                    struct_name: sname.clone(),
                                    field: field.clone(),
                                    line: *line,
                                    column: *column,
                                })?;

                        if !self.is_compatible(&val_type, expected_type) {
                            return Err(CompilerError::TypeMismatch {
                                operation: format!("struct field '{}' assignment", field),
                                expected: expected_type.to_string(),
                                found: val_type.to_string(),
                                line: *line,
                                column: *column,
                            });
                        }
                    }
                    Type::Class(ref cname) => {
                        let mut declared_field_type = None;
                        let mut curr = Some(cname.clone());
                        while let Some(curr_name) = curr {
                            if let Some(fields) = self.class_fields.get(&curr_name) {
                                if let Some(ty) = fields.get(field) {
                                    declared_field_type = Some(ty.clone());
                                    break;
                                }
                            }
                            curr = self.class_extends.get(&curr_name).and_then(|p| p.clone());
                        }

                        if let Some(expected_type) = declared_field_type {
                            if !self.is_compatible(&val_type, &expected_type) {
                                return Err(CompilerError::TypeMismatch {
                                    operation: format!("class field '{}' assignment", field),
                                    expected: expected_type.to_string(),
                                    found: val_type.to_string(),
                                    line: *line,
                                    column: *column,
                                });
                            }
                        } else {
                            return Err(CompilerError::NoSuchField {
                                struct_name: cname.clone(),
                                field: field.clone(),
                                line: *line,
                                column: *column,
                            });
                        }
                    }
                    Type::Any => {}
                    other => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "field assignment".to_string(),
                            expected: "Struct or Object".to_string(),
                            found: other.to_string(),
                            line: *line,
                            column: *column,
                        });
                    }
                }
                Ok(())
            }
            Statement::TryCatch {
                try_block,
                catch_var,
                catch_block,
                ..
            } => {
                let parent_env = self.env.clone();

                self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));
                self.check_block(try_block)?;

                self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));
                self.env.borrow_mut().define(catch_var.clone(), Type::Any);
                self.check_block(catch_block)?;

                self.env = parent_env;
                Ok(())
            }
            Statement::Throw { value, .. } => {
                self.check_expr(value)?;
                Ok(())
            }
            Statement::ImportDeclaration {
                name,
                path,
                line,
                column,
            } => {
                let builtins = ["math", "json", "http", "db", "os"];
                if builtins.contains(&name.as_str()) {
                    self.env.borrow_mut().define(name.clone(), Type::Any);
                    return Ok(());
                }

                let candidates: Vec<String> = if let Some(p) = path {
                    vec![if p.ends_with(".bz") {
                        p.clone()
                    } else {
                        format!("{p}.bz")
                    }]
                } else {
                    vec![format!("{name}.bz"), format!("stdlib/{name}.bz")]
                };

                let source = candidates
                    .iter()
                    .find_map(|file_path| std::fs::read_to_string(file_path).ok())
                    .ok_or_else(|| CompilerError::ModuleNotFound {
                        name: name.clone(),
                        line: *line,
                        column: *column,
                    })?;

                let tokens = crate::lexer::tokenize(&source)?;
                let program = crate::parser::parse(tokens)?;

                crate::typechecker::check(&program)?;

                self.env.borrow_mut().define(name.clone(), Type::Any);
                Ok(())
            }
            Statement::ExportDeclaration { declaration, .. } => {
                if let Some(decl) = declaration {
                    self.check_statement(decl)?;
                }
                Ok(())
            }
            Statement::EnumDeclaration { name, .. } => {
                self.env
                    .borrow_mut()
                    .define(name.clone(), Type::Class(name.clone()));
                Ok(())
            }
            Statement::MatchStatement { subject, arms, .. } => {
                let sub_type = self.check_expr(subject)?;
                let parent_env = self.env.clone();
                for arm in arms {
                    self.env = Rc::new(RefCell::new(TypeEnv::with_parent(parent_env.clone())));
                    self.bind_match_pattern(&arm.pattern, &sub_type)?;
                    self.check_block(&arm.body)?;
                }
                self.env = parent_env;
                Ok(())
            }
            Statement::SpawnStatement { expression, .. } => {
                self.check_expr(expression)?;
                Ok(())
            }
            Statement::InterfaceDeclaration { .. } => Ok(()),
        }
    }

    fn bind_match_pattern(
        &mut self,
        pattern: &MatchPattern,
        ty: &Type,
    ) -> Result<(), CompilerError> {
        match pattern {
            MatchPattern::Wildcard
            | MatchPattern::Integer(_)
            | MatchPattern::Float(_)
            | MatchPattern::StringLit(_)
            | MatchPattern::Boolean(_)
            | MatchPattern::Null => Ok(()),
            MatchPattern::Identifier(name) => {
                if !self.class_fields.contains_key(name) && !self.struct_defs.contains_key(name) {
                    self.env.borrow_mut().define(name.clone(), ty.clone());
                }
                Ok(())
            }
            MatchPattern::EnumVariant(_, payload) => {
                if let Some(p) = payload {
                    self.bind_match_pattern(p, &Type::Any)?;
                }
                Ok(())
            }
        }
    }

    fn check_block(&mut self, statements: &[Statement]) -> Result<(), CompilerError> {
        for stmt in statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }
}

/// The main entry point for static type checking.
pub fn check(program: &Program) -> Result<(), CompilerError> {
    let mut checker = TypeChecker {
        env: Rc::new(RefCell::new(TypeEnv::new())),
        struct_defs: HashMap::new(),
        class_fields: HashMap::new(),
        class_methods: HashMap::new(),
        class_extends: HashMap::new(),
        class_implements: HashMap::new(),
        interface_methods: HashMap::new(),
        expected_return_type: None,
    };

    // Register built-in functions
    {
        let mut e = checker.env.borrow_mut();
        let builtins = vec![
            "len", "type", "str", "to_int", "to_float", "input", "push", "pop", "keys", "values",
            "contains", "split", "join", "map_fn", "map_set", "map_get", "channel", "send", "recv",
        ];
        for name in builtins {
            e.define(name.to_string(), Type::Any);
        }
    }

    checker.collect_declarations(&program.statements)?;
    checker.check_block(&program.statements)?;

    Ok(())
}
