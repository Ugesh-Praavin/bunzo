use std::collections::HashMap;
use crate::ast::{Expression, Program, Statement, MatchPattern, MatchArm};
use crate::lexer::tokenize;
use crate::parser::parse;
use crate::semantic::analyze;
use crate::typechecker::check as typecheck;
use crate::diagnostics::CompilerError;
use super::protocol::{Diagnostic, Range, Position, Hover, MarkupContent, Location};

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolInfo {
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub detail: String,
}

pub struct SymbolFinder<'a> {
    target_line: usize,
    target_col: usize,
    text: &'a str,
    scopes: Vec<HashMap<String, SymbolInfo>>,
    pub found_symbol: Option<SymbolInfo>,
}

fn find_identifier_column(text: &str, line_num: usize, name: &str) -> usize {
    if line_num == 0 {
        return 1;
    }
    if let Some(line) = text.lines().nth(line_num - 1) {
        if let Some(offset) = line.find(name) {
            return offset + 1; // 1-based column
        }
    }
    1
}

impl<'a> SymbolFinder<'a> {
    pub fn new(target_line: usize, target_col: usize, text: &'a str) -> Self {
        let mut scopes = vec![HashMap::new()];
        let builtins = vec![
            "len", "type", "str", "to_int", "to_float", "input", "push", "pop", "keys", "values",
            "contains", "split", "join", "map_fn", "map_set", "map_get", "channel", "send", "recv"
        ];
        for b in builtins {
            scopes[0].insert(b.to_string(), SymbolInfo {
                name: b.to_string(),
                line: 0,
                column: 0,
                detail: format!("built-in function {}", b),
            });
        }
        SymbolFinder {
            target_line,
            target_col,
            text,
            scopes,
            found_symbol: None,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: String, line: usize, col: usize, detail: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), SymbolInfo {
                name,
                line,
                column: col,
                detail,
            });
        }
    }

    fn lookup(&self, name: &str) -> Option<SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info.clone());
            }
        }
        None
    }

    fn check_identifier(&mut self, name: &str, line: usize, col: usize) {
        if self.target_line == line && self.target_col >= col && self.target_col < col + name.len() {
            if let Some(info) = self.lookup(name) {
                self.found_symbol = Some(info);
            }
        }
    }

    pub fn walk_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.collect_top_level_definitions(stmt);
        }
        for stmt in &program.statements {
            self.walk_statement(stmt);
        }
    }

    fn collect_top_level_definitions(&mut self, stmt: &Statement) {
        match stmt {
            Statement::FunctionDeclaration { name, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.define(name.clone(), *line, col, format!("fn {}(...)", name));
            }
            Statement::StructDeclaration { name, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.define(name.clone(), *line, col, format!("struct {}", name));
            }
            Statement::ClassDeclaration { name, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.define(name.clone(), *line, col, format!("class {}", name));
            }
            Statement::InterfaceDeclaration { name, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.define(name.clone(), *line, col, format!("interface {}", name));
            }
            Statement::EnumDeclaration { name, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.define(name.clone(), *line, col, format!("enum {}", name));
            }
            Statement::ExportDeclaration { declaration, .. } => {
                if let Some(decl) = declaration {
                    self.collect_top_level_definitions(decl);
                }
            }
            _ => {}
        }
    }

    fn walk_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::LetDeclaration { name, initializer, line, .. } => {
                self.walk_expression(initializer);
                let col = find_identifier_column(self.text, *line, name);
                self.define(name.clone(), *line, col, format!("let {}", name));
                self.check_identifier(name, *line, col);
            }
            Statement::ConstDeclaration { name, initializer, line, .. } => {
                self.walk_expression(initializer);
                let col = find_identifier_column(self.text, *line, name);
                self.define(name.clone(), *line, col, format!("const {}", name));
                self.check_identifier(name, *line, col);
            }
            Statement::PrintStatement { argument, .. } => {
                self.walk_expression(argument);
            }
            Statement::ExpressionStatement { expression } => {
                self.walk_expression(expression);
            }
            Statement::FunctionDeclaration { name, params, body, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.check_identifier(name, *line, col);

                self.push_scope();
                for param in params {
                    let p_col = find_identifier_column(self.text, param.line, &param.name);
                    self.define(
                        param.name.clone(),
                        param.line,
                        p_col,
                        format!("param {}: {}", param.name, param.type_name),
                    );
                    self.check_identifier(&param.name, param.line, p_col);
                }
                for sub_stmt in body {
                    self.walk_statement(sub_stmt);
                }
                self.pop_scope();
            }
            Statement::ReturnStatement { value, .. } => {
                if let Some(expr) = value {
                    self.walk_expression(expr);
                }
            }
            Statement::Assignment { name, value, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.check_identifier(name, *line, col);
                self.walk_expression(value);
            }
            Statement::IfStatement { condition, then_branch, else_branch, .. } => {
                self.walk_expression(condition);
                self.push_scope();
                for s in then_branch {
                    self.walk_statement(s);
                }
                self.pop_scope();
                if let Some(branch) = else_branch {
                    self.push_scope();
                    for s in branch {
                        self.walk_statement(s);
                    }
                    self.pop_scope();
                }
            }
            Statement::WhileStatement { condition, body, .. } => {
                self.walk_expression(condition);
                self.push_scope();
                for s in body {
                    self.walk_statement(s);
                }
                self.pop_scope();
            }
            Statement::ForStatement { variable, start, end, body, line, .. } => {
                self.walk_expression(start);
                self.walk_expression(end);
                self.push_scope();
                let col = find_identifier_column(self.text, *line, variable);
                self.define(variable.clone(), *line, col, format!("for-loop variable {}", variable));
                self.check_identifier(variable, *line, col);
                for s in body {
                    self.walk_statement(s);
                }
                self.pop_scope();
            }
            Statement::BreakStatement { .. } | Statement::ContinueStatement { .. } => {}
            Statement::StructDeclaration { name, fields, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.check_identifier(name, *line, col);
                for f in fields {
                    let f_col = find_identifier_column(self.text, f.line, &f.name);
                    self.check_identifier(&f.name, f.line, f_col);
                }
            }
            Statement::ClassDeclaration { name, fields, methods, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.check_identifier(name, *line, col);
                self.push_scope();
                for f in fields {
                    let f_col = find_identifier_column(self.text, f.line, &f.name);
                    self.define(f.name.clone(), f.line, f_col, format!("field {}: {}", f.name, f.type_name));
                    self.check_identifier(&f.name, f.line, f_col);
                }
                for m in methods {
                    self.walk_statement(m);
                }
                self.pop_scope();
            }
            Statement::FieldAssignment { object, field, value, line, .. } => {
                self.walk_expression(object);
                let col = find_identifier_column(self.text, *line, field);
                self.check_identifier(field, *line, col);
                self.walk_expression(value);
            }
            Statement::TryCatch { try_block, catch_var, catch_block, line, .. } => {
                self.push_scope();
                for s in try_block {
                    self.walk_statement(s);
                }
                self.pop_scope();

                self.push_scope();
                let col = find_identifier_column(self.text, *line, catch_var);
                self.define(catch_var.clone(), *line, col, format!("exception {}", catch_var));
                self.check_identifier(catch_var, *line, col);
                for s in catch_block {
                    self.walk_statement(s);
                }
                self.pop_scope();
            }
            Statement::Throw { value, .. } => {
                self.walk_expression(value);
            }
            Statement::ImportDeclaration { name, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.check_identifier(name, *line, col);
            }
            Statement::ExportDeclaration { declaration, .. } => {
                if let Some(decl) = declaration {
                    self.walk_statement(decl);
                }
            }
            Statement::EnumDeclaration { name, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.check_identifier(name, *line, col);
            }
            Statement::MatchStatement { subject, arms, .. } => {
                self.walk_expression(subject);
                for arm in arms {
                    self.walk_match_arm(arm);
                }
            }
            Statement::InterfaceDeclaration { name, methods, line, .. } => {
                let col = find_identifier_column(self.text, *line, name);
                self.check_identifier(name, *line, col);
                for m in methods {
                    let m_col = find_identifier_column(self.text, m.line, &m.name);
                    self.check_identifier(&m.name, m.line, m_col);
                }
            }
            Statement::SpawnStatement { expression, .. } => {
                self.walk_expression(expression);
            }
        }
    }

    fn walk_match_arm(&mut self, arm: &MatchArm) {
        self.push_scope();
        self.walk_match_pattern(&arm.pattern);
        for s in &arm.body {
            self.walk_statement(s);
        }
        self.pop_scope();
    }

    fn walk_match_pattern(&mut self, pattern: &MatchPattern) {
        match pattern {
            MatchPattern::Identifier(name) => {
                self.define(name.clone(), 0, 0, format!("matched variable {}", name));
            }
            MatchPattern::EnumVariant(_, payload) => {
                if let Some(p) = payload {
                    self.walk_match_pattern(p);
                }
            }
            _ => {}
        }
    }

    fn walk_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BooleanLiteral { .. }
            | Expression::NullLiteral { .. } => {}
            Expression::Identifier { name, line, column } => {
                self.check_identifier(name, *line, *column);
            }
            Expression::BinaryOp { left, right, .. } => {
                self.walk_expression(left);
                self.walk_expression(right);
            }
            Expression::UnaryOp { operand, .. } => {
                self.walk_expression(operand);
            }
            Expression::Grouping { expression, .. } => {
                self.walk_expression(expression);
            }
            Expression::Call { callee, arguments, .. } => {
                self.walk_expression(callee);
                for arg in arguments {
                    self.walk_expression(arg);
                }
            }
            Expression::StructLiteral { name, fields, line, column } => {
                self.check_identifier(name, *line, *column);
                for (_, val) in fields {
                    self.walk_expression(val);
                }
            }
            Expression::FieldAccess { object, field, line, column } => {
                self.walk_expression(object);
                self.check_identifier(field, *line, *column);
            }
            Expression::ArrayLiteral { elements, .. } => {
                for el in elements {
                    self.walk_expression(el);
                }
            }
            Expression::IndexExpression { object, index, .. } => {
                self.walk_expression(object);
                self.walk_expression(index);
            }
            Expression::EnumVariantExpr { payload, .. } => {
                if let Some(p) = payload {
                    self.walk_expression(p);
                }
            }
            Expression::PropagateError { expression, .. } => {
                self.walk_expression(expression);
            }
            Expression::MoveExpr { name, line, column } => {
                self.check_identifier(name, *line, *column);
            }
            Expression::AwaitExpr { expression, .. } => {
                self.walk_expression(expression);
            }
            Expression::SuperExpr { .. } => {}
        }
    }
}

pub fn compile_and_get_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let tokens = match tokenize(text) {
        Ok(t) => t,
        Err(e) => {
            diagnostics.push(convert_error_to_diagnostic(e));
            return diagnostics;
        }
    };

    let program = match parse(tokens) {
        Ok(p) => p,
        Err(e) => {
            diagnostics.push(convert_error_to_diagnostic(e));
            return diagnostics;
        }
    };

    if let Err(e) = analyze(&program) {
        diagnostics.push(convert_error_to_diagnostic(e));
        return diagnostics;
    }

    if let Err(e) = typecheck(&program) {
        diagnostics.push(convert_error_to_diagnostic(e));
        return diagnostics;
    }

    diagnostics
}

fn convert_error_to_diagnostic(err: CompilerError) -> Diagnostic {
    let msg = format!("{}", err);
    let (line, col) = err.location().unwrap_or((1, 1));
    
    let start_pos = Position {
        line: (line as u32).saturating_sub(1),
        character: (col as u32).saturating_sub(1),
    };
    
    let end_pos = Position {
        line: start_pos.line,
        character: start_pos.character + 1,
    };

    Diagnostic {
        range: Range { start: start_pos, end: end_pos },
        severity: Some(1),
        message: msg,
    }
}

pub fn handle_hover(text: &str, line: u32, character: u32) -> Option<Hover> {
    let tokens = tokenize(text).ok()?;
    let program = parse(tokens).ok()?;
    
    let target_line = (line as usize) + 1;
    let target_col = (character as usize) + 1;

    let mut finder = SymbolFinder::new(target_line, target_col, text);
    finder.walk_program(&program);

    if let Some(info) = finder.found_symbol {
        Some(Hover {
            contents: MarkupContent {
                kind: "markdown".to_string(),
                value: format!("```bunzo\n{}\n```", info.detail),
            },
        })
    } else {
        None
    }
}

pub fn handle_definition(uri: &str, text: &str, line: u32, character: u32) -> Option<Location> {
    let tokens = tokenize(text).ok()?;
    let program = parse(tokens).ok()?;
    
    let target_line = (line as usize) + 1;
    let target_col = (character as usize) + 1;

    let mut finder = SymbolFinder::new(target_line, target_col, text);
    finder.walk_program(&program);

    if let Some(info) = finder.found_symbol {
        if info.line == 0 {
            return None;
        }
        let start_pos = Position {
            line: (info.line as u32).saturating_sub(1),
            character: (info.column as u32).saturating_sub(1),
        };
        let end_pos = Position {
            line: start_pos.line,
            character: start_pos.character + (info.name.len() as u32),
        };
        Some(Location {
            uri: uri.to_string(),
            range: Range { start: start_pos, end: end_pos },
        })
    } else {
        None
    }
}
