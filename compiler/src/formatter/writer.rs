//! AST pretty printer and formatting visitor for Bunzo.

use crate::ast::{
    BinaryOperator, Expression, MatchArm, MatchPattern, Parameter, Program, Statement,
    UnaryOperator, Visibility,
};
use crate::lexer::{Token, TokenKind};
use crate::formatter::style::Config;

#[derive(Debug, Clone)]
pub struct Comment {
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub is_block: bool,
}

/// Lightweight comment extractor scanner.
pub fn extract_comments(source: &str) -> Vec<Comment> {
    let mut comments = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut pos = 0;
    let mut line = 1;
    let mut column = 1;

    let mut in_string = false;

    while pos < chars.len() {
        let c = chars[pos];
        if in_string {
            if c == '"' {
                in_string = false;
                pos += 1;
                column += 1;
            } else if c == '\n' {
                line += 1;
                column = 1;
                pos += 1;
            } else {
                pos += 1;
                column += 1;
            }
        } else {
            if c == '"' {
                in_string = true;
                pos += 1;
                column += 1;
            } else if c == '/' && pos + 1 < chars.len() && chars[pos + 1] == '/' {
                let start_line = line;
                let start_col = column;
                let mut text = String::new();
                text.push('/');
                text.push('/');
                pos += 2;
                column += 2;
                while pos < chars.len() && chars[pos] != '\n' && chars[pos] != '\r' {
                    text.push(chars[pos]);
                    pos += 1;
                    column += 1;
                }
                comments.push(Comment {
                    line: start_line,
                    column: start_col,
                    text,
                    is_block: false,
                });
            } else if c == '/' && pos + 1 < chars.len() && chars[pos + 1] == '*' {
                let start_line = line;
                let start_col = column;
                let mut text = String::new();
                text.push('/');
                text.push('*');
                pos += 2;
                column += 2;
                while pos < chars.len() {
                    if chars[pos] == '*' && pos + 1 < chars.len() && chars[pos + 1] == '/' {
                        text.push('*');
                        text.push('/');
                        pos += 2;
                        column += 2;
                        break;
                    }
                    let bc = chars[pos];
                    text.push(bc);
                    if bc == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    pos += 1;
                }
                comments.push(Comment {
                    line: start_line,
                    column: start_col,
                    text,
                    is_block: true,
                });
            } else if c == '\n' {
                line += 1;
                column = 1;
                pos += 1;
            } else {
                pos += 1;
                column += 1;
            }
        }
    }
    comments
}

/// Maps positions of `{` to their matching `}`.
pub fn map_braces(tokens: &[Token]) -> std::collections::HashMap<(usize, usize), (usize, usize)> {
    let mut stack = Vec::new();
    let mut brace_pairs = std::collections::HashMap::new();
    for token in tokens {
        if token.kind == TokenKind::LeftBrace {
            stack.push((token.line, token.column));
        } else if token.kind == TokenKind::RightBrace {
            if let Some(left) = stack.pop() {
                brace_pairs.insert(left, (token.line, token.column));
            }
        }
    }
    brace_pairs
}

pub struct Formatter<'a> {
    tokens: &'a [Token],
    comments: Vec<Comment>,
    comment_idx: usize,
    brace_pairs: std::collections::HashMap<(usize, usize), (usize, usize)>,
    config: Config,
    output: String,
    indent_level: usize,
    line_start: bool,
    last_search_line: usize,
}

impl<'a> Formatter<'a> {
    pub fn new(
        tokens: &'a [Token],
        comments: Vec<Comment>,
        brace_pairs: std::collections::HashMap<(usize, usize), (usize, usize)>,
        config: Config,
    ) -> Self {
        Self {
            tokens,
            comments,
            comment_idx: 0,
            brace_pairs,
            config,
            output: String::new(),
            indent_level: 0,
            line_start: true,
            last_search_line: 0,
        }
    }

    pub fn format(mut self, prog: &Program) -> String {
        for (i, stmt) in prog.statements.iter().enumerate() {
            let start_line = stmt_line(stmt);
            let start_col = stmt_column(stmt);
            self.print_comments_before(start_line, start_col);

            if i > 0 && is_top_level_declaration(stmt) {
                self.writeln();
            }

            self.format_statement(stmt);
            self.print_trailing_comment(self.stmt_last_line(stmt));
            self.writeln();
        }
        self.print_all_remaining_comments();

        clean_trailing_whitespace(&self.output, self.config.newline_style)
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn write(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        if self.line_start {
            self.output.push_str(&" ".repeat(self.indent_level * self.config.indent));
            self.line_start = false;
        }
        self.output.push_str(s);
    }

    fn writeln(&mut self) {
        self.output.push_str(self.config.newline_style);
        self.line_start = true;
    }

    fn write_space(&mut self) {
        if !self.line_start
            && !self.output.ends_with(' ')
            && !self.output.ends_with('\n')
            && !self.output.ends_with('\r')
        {
            self.output.push(' ');
        }
    }

    fn print_comments_before(&mut self, line: usize, col: usize) {
        while self.comment_idx < self.comments.len() {
            let comment = &self.comments[self.comment_idx];
            if comment.line < line || (comment.line == line && comment.column < col) {
                let text = comment.text.clone();
                if !self.line_start {
                    self.writeln();
                }
                self.write(&text);
                self.writeln();
                self.comment_idx += 1;
            } else {
                break;
            }
        }
    }

    fn print_trailing_comment(&mut self, line: usize) {
        while self.comment_idx < self.comments.len() {
            let comment = &self.comments[self.comment_idx];
            if comment.line == line {
                let text = comment.text.clone();
                self.write_space();
                self.write(&text);
                self.comment_idx += 1;
                break;
            } else {
                break;
            }
        }
    }

    fn print_all_remaining_comments(&mut self) {
        while self.comment_idx < self.comments.len() {
            let text = self.comments[self.comment_idx].text.clone();
            if !self.line_start {
                self.writeln();
            }
            self.write(&text);
            self.writeln();
            self.comment_idx += 1;
        }
    }

    fn has_comments_before(&self, line: usize, col: usize) -> bool {
        if self.comment_idx < self.comments.len() {
            let comment = &self.comments[self.comment_idx];
            comment.line < line || (comment.line == line && comment.column < col)
        } else {
            false
        }
    }

    fn find_block_end_line_after(&mut self, line: usize) -> usize {
        let search_line = std::cmp::max(line, self.last_search_line);
        let mut best_brace = None;
        for token in self.tokens {
            if token.line > search_line || (token.line == search_line && token.column >= 1) {
                if token.kind == TokenKind::LeftBrace {
                    best_brace = Some((token.line, token.column));
                    break;
                }
            }
        }
        let end_line = if let Some(left) = best_brace {
            if let Some(right) = self.brace_pairs.get(&left) {
                right.0
            } else {
                line
            }
        } else {
            line
        };
        self.last_search_line = end_line;
        end_line
    }

    fn stmt_last_line(&self, stmt: &Statement) -> usize {
        match stmt {
            Statement::IfStatement { line, .. }
            | Statement::WhileStatement { line, .. }
            | Statement::ForStatement { line, .. }
            | Statement::StructDeclaration { line, .. }
            | Statement::ClassDeclaration { line, .. }
            | Statement::InterfaceDeclaration { line, .. }
            | Statement::EnumDeclaration { line, .. }
            | Statement::MatchStatement { line, .. } => {
                // Find LeftBrace after start line
                let mut best_brace = None;
                for token in self.tokens {
                    if token.line >= *line {
                        if token.kind == TokenKind::LeftBrace {
                            best_brace = Some((token.line, token.column));
                            break;
                        }
                    }
                }
                if let Some(left) = best_brace {
                    if let Some(right) = self.brace_pairs.get(&left) {
                        return right.0;
                    }
                }
                *line
            }
            Statement::FunctionDeclaration { line, is_abstract, .. } => {
                if *is_abstract {
                    *line
                } else {
                    let mut best_brace = None;
                    for token in self.tokens {
                        if token.line >= *line {
                            if token.kind == TokenKind::LeftBrace {
                                best_brace = Some((token.line, token.column));
                                break;
                            }
                        }
                    }
                    if let Some(left) = best_brace {
                        if let Some(right) = self.brace_pairs.get(&left) {
                            return right.0;
                        }
                    }
                    *line
                }
            }
            Statement::TryCatch { line, .. } => {
                let mut left_braces = Vec::new();
                for token in self.tokens {
                    if token.line >= *line {
                        if token.kind == TokenKind::LeftBrace {
                            left_braces.push((token.line, token.column));
                        }
                    }
                }
                if left_braces.len() >= 2 {
                    if let Some(right) = self.brace_pairs.get(&left_braces[1]) {
                        return right.0;
                    }
                }
                *line
            }
            _ => stmt_line(stmt),
        }
    }

    fn format_block(&mut self, statements: &[Statement], node_line: usize) -> usize {
        self.write(" {");
        if statements.is_empty() {
            let end_line = self.find_block_end_line_after(node_line);
            if self.has_comments_before(end_line, 1) {
                self.indent();
                self.writeln();
                self.print_comments_before(end_line, 1);
                self.dedent();
                self.write("}");
            } else {
                self.write("}");
            }
            end_line
        } else {
            self.indent();
            self.writeln();

            for stmt in statements {
                let line = stmt_line(stmt);
                let col = stmt_column(stmt);
                self.print_comments_before(line, col);

                self.format_statement(stmt);
                self.print_trailing_comment(self.stmt_last_line(stmt));
                self.writeln();
            }

            let end_line = self.find_block_end_line_after(node_line);
            self.print_comments_before(end_line, 1);

            self.dedent();
            self.write("}");
            end_line
        }
    }

    fn format_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::LetDeclaration { name, initializer, .. } => {
                self.write("let ");
                self.write(name);
                self.write(" = ");
                self.format_expression(initializer);
            }
            Statement::ConstDeclaration { name, initializer, .. } => {
                self.write("const ");
                self.write(name);
                self.write(" = ");
                self.format_expression(initializer);
            }
            Statement::PrintStatement { argument, .. } => {
                self.write("print(");
                self.format_expression(argument);
                self.write(")");
            }
            Statement::ExpressionStatement { expression } => {
                self.format_expression(expression);
            }
            Statement::FunctionDeclaration {
                name,
                params,
                return_type,
                body,
                visibility,
                is_abstract,
                line,
                ..
            } => {
                if *visibility == Visibility::Private {
                    self.write("private ");
                }
                if *is_abstract {
                    self.write("abstract ");
                }
                self.write("func ");
                self.write(name);
                self.write("(");
                self.format_parameters(params);
                self.write(")");
                if let Some(rt) = return_type {
                    self.write(" -> ");
                    self.write(rt);
                }
                if *is_abstract {
                    // Abstract methods have no body.
                } else {
                    self.format_block(body, *line);
                }
            }
            Statement::ReturnStatement { value, .. } => {
                self.write("return");
                if let Some(expr) = value {
                    self.write(" ");
                    self.format_expression(expr);
                }
            }
            Statement::Assignment { name, value, .. } => {
                self.write(name);
                self.write(" = ");
                self.format_expression(value);
            }
            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                line,
                ..
            } => {
                self.write("if ");
                self.format_expression(condition);
                let then_end = self.format_block(then_branch, *line);
                if let Some(else_stmts) = else_branch {
                    self.write(" else");
                    if else_stmts.len() == 1 {
                        if let Statement::IfStatement { .. } = &else_stmts[0] {
                            self.write(" ");
                            self.format_statement(&else_stmts[0]);
                            return;
                        }
                    }
                    self.format_block(else_stmts, then_end);
                }
            }
            Statement::WhileStatement {
                condition,
                body,
                line,
                ..
            } => {
                self.write("while ");
                self.format_expression(condition);
                self.format_block(body, *line);
            }
            Statement::ForStatement {
                variable,
                start,
                end,
                body,
                line,
                ..
            } => {
                self.write("for ");
                self.write(variable);
                self.write(" in ");
                self.format_expression(start);
                self.write("..");
                self.format_expression(end);
                self.format_block(body, *line);
            }
            Statement::BreakStatement { .. } => {
                self.write("break");
            }
            Statement::ContinueStatement { .. } => {
                self.write("continue");
            }
            Statement::StructDeclaration { name, fields, line, .. } => {
                self.write("struct ");
                self.write(name);
                self.write(" {");
                if fields.is_empty() {
                    self.write("}");
                } else {
                    self.indent();
                    self.writeln();
                    for field in fields {
                        self.print_comments_before(field.line, field.column);
                        self.write(&field.name);
                        self.write(": ");
                        self.write(&field.type_name);
                        self.print_trailing_comment(field.line);
                        self.writeln();
                    }
                    let end_line = self.find_block_end_line_after(*line);
                    self.print_comments_before(end_line, 1);
                    self.dedent();
                    self.write("}");
                }
            }
            Statement::ClassDeclaration {
                name,
                extends,
                implements,
                is_abstract,
                fields,
                methods,
                line,
                ..
            } => {
                if *is_abstract {
                    self.write("abstract ");
                }
                self.write("class ");
                self.write(name);
                if let Some(ext) = extends {
                    self.write(" extends ");
                    self.write(ext);
                }
                if !implements.is_empty() {
                    self.write(" implements ");
                    for (i, impl_name) in implements.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write(impl_name);
                    }
                }
                self.write(" {");
                self.indent();
                self.writeln();

                for field in fields {
                    self.print_comments_before(field.line, field.column);
                    if field.visibility == Visibility::Private {
                        self.write("private ");
                    }
                    self.write(&field.name);
                    self.write(": ");
                    self.write(&field.type_name);
                    self.print_trailing_comment(field.line);
                    self.writeln();
                }

                if !fields.is_empty() && !methods.is_empty() {
                    self.writeln();
                }

                for (i, method) in methods.iter().enumerate() {
                    if i > 0 {
                        self.writeln();
                    }
                    let m_line = stmt_line(method);
                    let m_col = stmt_column(method);
                    self.print_comments_before(m_line, m_col);
                    self.format_statement(method);
                    self.print_trailing_comment(self.stmt_last_line(method));
                    self.writeln();
                }

                let end_line = self.find_block_end_line_after(*line);
                self.print_comments_before(end_line, 1);
                self.dedent();
                self.write("}");
            }
            Statement::FieldAssignment {
                object,
                field,
                value,
                ..
            } => {
                self.format_expression(object);
                self.write(".");
                self.write(field);
                self.write(" = ");
                self.format_expression(value);
            }
            Statement::TryCatch {
                try_block,
                catch_var,
                catch_block,
                line,
                ..
            } => {
                self.write("try");
                let try_end = self.format_block(try_block, *line);
                self.write(" catch ");
                self.write(catch_var);
                self.format_block(catch_block, try_end);
            }
            Statement::Throw { value, .. } => {
                self.write("throw ");
                self.format_expression(value);
            }
            Statement::ImportDeclaration { name, path, .. } => {
                self.write("import ");
                self.write(name);
                if let Some(p) = path {
                    self.write(" from \"");
                    self.write(p);
                    self.write("\"");
                }
            }
            Statement::ExportDeclaration {
                name,
                declaration,
                ..
            } => {
                self.write("export ");
                if let Some(decl) = declaration {
                    self.format_statement(decl);
                } else {
                    self.write(name);
                }
            }
            Statement::EnumDeclaration {
                name,
                variants,
                line,
                ..
            } => {
                self.write("enum ");
                self.write(name);
                self.write(" {");
                self.indent();
                self.writeln();
                for (v_name, payload) in variants {
                    self.write(v_name);
                    if let Some(p_type) = payload {
                        self.write("(");
                        self.write(p_type);
                        self.write(")");
                    }
                    self.writeln();
                }
                let end_line = self.find_block_end_line_after(*line);
                self.print_comments_before(end_line, 1);
                self.dedent();
                self.write("}");
            }
            Statement::MatchStatement {
                subject,
                arms,
                line,
                ..
            } => {
                self.write("match ");
                self.format_expression(subject);
                self.write(" {");
                self.indent();
                self.writeln();
                for arm in arms {
                    self.format_match_arm(arm, *line);
                }
                let end_line = self.find_block_end_line_after(*line);
                self.print_comments_before(end_line, 1);
                self.dedent();
                self.write("}");
            }
            Statement::InterfaceDeclaration {
                name,
                methods,
                line,
                ..
            } => {
                self.write("interface ");
                self.write(name);
                self.write(" {");
                self.indent();
                self.writeln();
                for method in methods {
                    self.print_comments_before(method.line, method.column);
                    self.write("func ");
                    self.write(&method.name);
                    self.write("(");
                    self.format_parameters(&method.params);
                    self.write(")");
                    if let Some(ref rt) = method.return_type {
                        self.write(" -> ");
                        self.write(rt);
                    }
                    self.print_trailing_comment(method.line);
                    self.writeln();
                }
                let end_line = self.find_block_end_line_after(*line);
                self.print_comments_before(end_line, 1);
                self.dedent();
                self.write("}");
            }
            Statement::SpawnStatement { expression, .. } => {
                self.write("spawn ");
                self.format_expression(expression);
            }
        }
    }

    fn format_match_arm(&mut self, arm: &MatchArm, match_line: usize) {
        self.format_pattern(&arm.pattern);
        self.write(" =>");
        self.format_block(&arm.body, match_line);
        self.writeln();
    }

    fn format_pattern(&mut self, pattern: &MatchPattern) {
        match pattern {
            MatchPattern::Wildcard => self.write("_"),
            MatchPattern::Integer(val) => self.write(&val.to_string()),
            MatchPattern::Float(val) => self.write(&val.to_string()),
            MatchPattern::StringLit(val) => {
                self.write("\"");
                self.write(val);
                self.write("\"");
            }
            MatchPattern::Boolean(val) => self.write(if *val { "true" } else { "false" }),
            MatchPattern::Null => self.write("null"),
            MatchPattern::Identifier(name) => self.write(name),
            MatchPattern::EnumVariant(name, payload) => {
                self.write(name);
                if let Some(p) = payload {
                    self.write("(");
                    self.format_pattern(p);
                    self.write(")");
                }
            }
        }
    }

    fn format_parameters(&mut self, params: &[Parameter]) {
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(&param.name);
            self.write(": ");
            self.write(&param.type_name);
        }
    }

    fn format_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::IntegerLiteral { value, .. } => {
                self.write(&value.to_string());
            }
            Expression::FloatLiteral { value, .. } => {
                self.write(&value.to_string());
            }
            Expression::StringLiteral { value, .. } => {
                self.write("\"");
                self.write(value);
                self.write("\"");
            }
            Expression::BooleanLiteral { value, .. } => {
                self.write(if *value { "true" } else { "false" });
            }
            Expression::NullLiteral { .. } => {
                self.write("null");
            }
            Expression::Identifier { name, .. } => {
                self.write(name);
            }
            Expression::SuperExpr { .. } => {
                self.write("super");
            }
            Expression::UnaryOp {
                operator,
                operand,
                ..
            } => {
                match operator {
                    UnaryOperator::Negate => self.write("-"),
                    UnaryOperator::LogicalNot => self.write("!"),
                }
                self.format_expression(operand);
            }
            Expression::BinaryOp {
                operator,
                left,
                right,
                ..
            } => {
                self.format_expression(left);
                self.write(" ");
                let op_str = match operator {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Modulo => "%",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::Less => "<",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                };
                self.write(op_str);
                self.write(" ");
                self.format_expression(right);
            }
            Expression::Grouping { expression, .. } => {
                self.write("(");
                self.format_expression(expression);
                self.write(")");
            }
            Expression::Call {
                callee,
                arguments,
                ..
            } => {
                self.format_expression(callee);
                self.write("(");
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_expression(arg);
                }
                self.write(")");
            }
            Expression::StructLiteral { name, fields, .. } => {
                self.write(name);
                self.write(" {");
                if fields.is_empty() {
                    self.write("}");
                } else {
                    self.indent();
                    self.writeln();
                    for (i, (f_name, f_val)) in fields.iter().enumerate() {
                        self.write(f_name);
                        self.write(": ");
                        self.format_expression(f_val);
                        if i + 1 < fields.len() {
                            self.write(",");
                        }
                        self.writeln();
                    }
                    self.dedent();
                    self.write("}");
                }
            }
            Expression::FieldAccess { object, field, .. } => {
                self.format_expression(object);
                self.write(".");
                self.write(field);
            }
            Expression::ArrayLiteral { elements, .. } => {
                self.write("[");
                for (i, el) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.format_expression(el);
                }
                self.write("]");
            }
            Expression::IndexExpression { object, index, .. } => {
                self.format_expression(object);
                self.write("[");
                self.format_expression(index);
                self.write("]");
            }
            Expression::EnumVariantExpr {
                enum_name,
                variant,
                payload,
                ..
            } => {
                self.write(enum_name);
                self.write("::");
                self.write(variant);
                if let Some(p) = payload {
                    self.write("(");
                    self.format_expression(p);
                    self.write(")");
                }
            }
            Expression::PropagateError { expression, .. } => {
                self.format_expression(expression);
                self.write("?");
            }
            Expression::MoveExpr { name, .. } => {
                self.write("move ");
                self.write(name);
            }
            Expression::AwaitExpr { expression, .. } => {
                self.write("await ");
                self.format_expression(expression);
            }
        }
    }
}

fn stmt_line(stmt: &Statement) -> usize {
    match stmt {
        Statement::LetDeclaration { line, .. } => *line,
        Statement::ConstDeclaration { line, .. } => *line,
        Statement::PrintStatement { line, .. } => *line,
        Statement::ExpressionStatement { expression } => expr_line(expression),
        Statement::FunctionDeclaration { line, .. } => *line,
        Statement::ReturnStatement { line, .. } => *line,
        Statement::Assignment { line, .. } => *line,
        Statement::IfStatement { line, .. } => *line,
        Statement::WhileStatement { line, .. } => *line,
        Statement::ForStatement { line, .. } => *line,
        Statement::BreakStatement { line, .. } => *line,
        Statement::ContinueStatement { line, .. } => *line,
        Statement::StructDeclaration { line, .. } => *line,
        Statement::ClassDeclaration { line, .. } => *line,
        Statement::FieldAssignment { line, .. } => *line,
        Statement::TryCatch { line, .. } => *line,
        Statement::Throw { line, .. } => *line,
        Statement::ImportDeclaration { line, .. } => *line,
        Statement::ExportDeclaration { line, .. } => *line,
        Statement::EnumDeclaration { line, .. } => *line,
        Statement::MatchStatement { line, .. } => *line,
        Statement::InterfaceDeclaration { line, .. } => *line,
        Statement::SpawnStatement { line, .. } => *line,
    }
}

fn stmt_column(stmt: &Statement) -> usize {
    match stmt {
        Statement::LetDeclaration { column, .. } => *column,
        Statement::ConstDeclaration { column, .. } => *column,
        Statement::PrintStatement { column, .. } => *column,
        Statement::ExpressionStatement { expression } => expr_column(expression),
        Statement::FunctionDeclaration { column, .. } => *column,
        Statement::ReturnStatement { column, .. } => *column,
        Statement::Assignment { column, .. } => *column,
        Statement::IfStatement { column, .. } => *column,
        Statement::WhileStatement { column, .. } => *column,
        Statement::ForStatement { column, .. } => *column,
        Statement::BreakStatement { column, .. } => *column,
        Statement::ContinueStatement { column, .. } => *column,
        Statement::StructDeclaration { column, .. } => *column,
        Statement::ClassDeclaration { column, .. } => *column,
        Statement::FieldAssignment { column, .. } => *column,
        Statement::TryCatch { column, .. } => *column,
        Statement::Throw { column, .. } => *column,
        Statement::ImportDeclaration { column, .. } => *column,
        Statement::ExportDeclaration { column, .. } => *column,
        Statement::EnumDeclaration { column, .. } => *column,
        Statement::MatchStatement { column, .. } => *column,
        Statement::InterfaceDeclaration { column, .. } => *column,
        Statement::SpawnStatement { column, .. } => *column,
    }
}

fn expr_line(expr: &Expression) -> usize {
    match expr {
        Expression::IntegerLiteral { line, .. } => *line,
        Expression::FloatLiteral { line, .. } => *line,
        Expression::StringLiteral { line, .. } => *line,
        Expression::BooleanLiteral { line, .. } => *line,
        Expression::NullLiteral { line, .. } => *line,
        Expression::Identifier { line, .. } => *line,
        Expression::BinaryOp { line, .. } => *line,
        Expression::UnaryOp { line, .. } => *line,
        Expression::Grouping { line, .. } => *line,
        Expression::Call { line, .. } => *line,
        Expression::StructLiteral { line, .. } => *line,
        Expression::FieldAccess { line, .. } => *line,
        Expression::ArrayLiteral { line, .. } => *line,
        Expression::IndexExpression { line, .. } => *line,
        Expression::EnumVariantExpr { line, .. } => *line,
        Expression::PropagateError { line, .. } => *line,
        Expression::MoveExpr { line, .. } => *line,
        Expression::AwaitExpr { line, .. } => *line,
        Expression::SuperExpr { line, .. } => *line,
    }
}

fn expr_column(expr: &Expression) -> usize {
    match expr {
        Expression::IntegerLiteral { column, .. } => *column,
        Expression::FloatLiteral { column, .. } => *column,
        Expression::StringLiteral { column, .. } => *column,
        Expression::BooleanLiteral { column, .. } => *column,
        Expression::NullLiteral { column, .. } => *column,
        Expression::Identifier { column, .. } => *column,
        Expression::BinaryOp { column, .. } => *column,
        Expression::UnaryOp { column, .. } => *column,
        Expression::Grouping { column, .. } => *column,
        Expression::Call { column, .. } => *column,
        Expression::StructLiteral { column, .. } => *column,
        Expression::FieldAccess { column, .. } => *column,
        Expression::ArrayLiteral { column, .. } => *column,
        Expression::IndexExpression { column, .. } => *column,
        Expression::EnumVariantExpr { column, .. } => *column,
        Expression::PropagateError { column, .. } => *column,
        Expression::MoveExpr { column, .. } => *column,
        Expression::AwaitExpr { column, .. } => *column,
        Expression::SuperExpr { column, .. } => *column,
    }
}

fn is_top_level_declaration(stmt: &Statement) -> bool {
    matches!(
        stmt,
        Statement::FunctionDeclaration { .. }
            | Statement::ClassDeclaration { .. }
            | Statement::StructDeclaration { .. }
            | Statement::EnumDeclaration { .. }
            | Statement::InterfaceDeclaration { .. }
    )
}

fn clean_trailing_whitespace(s: &str, newline: &str) -> String {
    let mut result = String::new();
    for line in s.lines() {
        result.push_str(line.trim_end());
        result.push_str(newline);
    }
    while result.ends_with(newline) {
        result.truncate(result.len() - newline.len());
    }
    if !result.is_empty() {
        result.push_str(newline);
    }
    result
}
