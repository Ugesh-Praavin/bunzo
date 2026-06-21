//! AST evaluation and execution engine for the Bunzo runtime.

use std::cell::RefCell;
use std::rc::Rc;

use super::environment::Environment;
use super::value::RuntimeValue;
use crate::ast::{BinaryOperator, Block, Expression, Program, Statement, UnaryOperator};
use crate::diagnostics::CompilerError;

// ── Control Flow Signal ───────────────────────────────────────────────────

/// The non-error outcome of executing a statement or block.
///
/// Used to propagate `break` and `continue` up the call stack without
/// treating them as errors.
#[derive(Debug, Clone, PartialEq)]
enum ControlFlow {
    /// Normal execution — keep going.
    None,
    /// A `break` was encountered — exit the nearest enclosing loop.
    Break,
    /// A `continue` was encountered — skip to the next iteration.
    Continue,
}

/// Executes a complete Bunzo program using the standard output.
///
/// # Errors
///
/// Returns a [`CompilerError`] if any runtime or evaluation error occurs.
pub fn execute(program: Program) -> Result<(), CompilerError> {
    let mut interpreter = Interpreter::new(std::io::stdout());
    interpreter.interpret(program)
}

/// The Bunzo AST interpreter.
pub struct Interpreter<W: std::io::Write> {
    /// The current lexical environment.
    environment: Rc<RefCell<Environment>>,
    /// The stream to write output from print statements.
    stdout: W,
}

impl<W: std::io::Write> Interpreter<W> {
    /// Creates a new interpreter writing output to the given stream.
    pub fn new(stdout: W) -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
            stdout,
        }
    }

    /// Interprets a complete program, executing statements sequentially.
    pub fn interpret(&mut self, program: Program) -> Result<(), CompilerError> {
        for stmt in program.statements {
            self.execute_statement(&stmt)?;
            // break/continue at top level are no-ops
        }
        Ok(())
    }

    /// Executes a block in a fresh child scope, propagating `ControlFlow`.
    ///
    /// Pushes a new environment onto the scope chain, runs each statement,
    /// then restores the previous environment. `Break` or `Continue` stops
    /// iteration and is returned to the caller.
    fn execute_block(&mut self, block: &Block) -> Result<ControlFlow, CompilerError> {
        let saved = Rc::clone(&self.environment);
        self.environment = Rc::new(RefCell::new(Environment::with_parent(Rc::clone(&saved))));

        let result = (|| {
            let mut flow = ControlFlow::None;
            for stmt in &block.statements {
                flow = self.execute_statement(stmt)?;
                if flow != ControlFlow::None {
                    break;
                }
            }
            Ok(flow)
        })();

        self.environment = saved;
        result
    }

    /// Executes a block in a child scope pre-seeded with a loop variable.
    ///
    /// Used by `for` loops to make the iteration variable available inside the body.
    fn execute_block_with_var(
        &mut self,
        block: &Block,
        var_name: &str,
        var_value: RuntimeValue,
        line: usize,
        column: usize,
    ) -> Result<ControlFlow, CompilerError> {
        let child_env = Rc::new(RefCell::new(Environment::with_parent(Rc::clone(
            &self.environment,
        ))));
        // Define loop variable in the child scope before executing the body.
        child_env
            .borrow_mut()
            .define(var_name.to_string(), var_value, false, line, column)?;

        let saved = Rc::clone(&self.environment);
        self.environment = child_env;

        let mut flow = ControlFlow::None;
        for stmt in &block.statements {
            flow = self.execute_statement(stmt)?;
            if flow != ControlFlow::None {
                break;
            }
        }

        self.environment = saved;
        Ok(flow)
    }

    // ── Statements ────────────────────────────────────────────────────────────

    fn execute_statement(&mut self, stmt: &Statement) -> Result<ControlFlow, CompilerError> {
        match stmt {
            Statement::LetDeclaration {
                name,
                initializer,
                line,
                column,
            } => {
                let value = self.evaluate_expression(initializer)?;
                self.environment.borrow_mut().define(
                    name.clone(),
                    value,
                    false, // is_const = false
                    *line,
                    *column,
                )?;
            }
            Statement::ConstDeclaration {
                name,
                initializer,
                line,
                column,
            } => {
                let value = self.evaluate_expression(initializer)?;
                self.environment.borrow_mut().define(
                    name.clone(),
                    value,
                    true, // is_const = true
                    *line,
                    *column,
                )?;
            }
            Statement::AssignStatement {
                name,
                value,
                line,
                column,
            } => {
                let new_val = self.evaluate_expression(value)?;
                self.environment
                    .borrow_mut()
                    .assign(name.clone(), new_val, *line, *column)?;
            }
            Statement::PrintStatement { argument, .. } => {
                let value = self.evaluate_expression(argument)?;
                writeln!(self.stdout, "{value}").map_err(CompilerError::Io)?;
            }
            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                line,
                column,
            } => {
                let cond_val = self.evaluate_expression(condition)?;
                let flow = match cond_val {
                    RuntimeValue::Boolean(true) => self.execute_block(then_branch)?,
                    RuntimeValue::Boolean(false) => {
                        if let Some(else_blk) = else_branch {
                            self.execute_block(else_blk)?
                        } else {
                            ControlFlow::None
                        }
                    }
                    other => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "if condition".to_string(),
                            expected: "Boolean".to_string(),
                            found: other.type_name().to_string(),
                            line: *line,
                            column: *column,
                        });
                    }
                };
                // Propagate break/continue out of if blocks (e.g. inside loops)
                if flow != ControlFlow::None {
                    return Ok(flow);
                }
            }
            Statement::WhileStatement {
                condition,
                body,
                line,
                column,
            } => loop {
                let cond_val = self.evaluate_expression(condition)?;
                match cond_val {
                    RuntimeValue::Boolean(true) => {}
                    RuntimeValue::Boolean(false) => break,
                    other => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "while condition".to_string(),
                            expected: "Boolean".to_string(),
                            found: other.type_name().to_string(),
                            line: *line,
                            column: *column,
                        });
                    }
                }
                match self.execute_block(body)? {
                    ControlFlow::Break => break,
                    ControlFlow::Continue => continue,
                    ControlFlow::None => {}
                }
            },
            Statement::ForInStatement {
                variable,
                iterable,
                body,
                line,
                column,
            } => {
                // Evaluate the range bounds.
                let (start, end, inclusive) = match self.evaluate_expression(iterable)? {
                    RuntimeValue::Range {
                        start,
                        end,
                        inclusive,
                    } => (start, end, inclusive),
                    other => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "for-in iterable".to_string(),
                            expected: "Range".to_string(),
                            found: other.type_name().to_string(),
                            line: *line,
                            column: *column,
                        });
                    }
                };
                let range_end = if inclusive { end + 1 } else { end };
                let mut i = start;
                while i < range_end {
                    let flow = self.execute_block_with_var(
                        body,
                        variable,
                        RuntimeValue::Integer(i),
                        *line,
                        *column,
                    )?;
                    match flow {
                        ControlFlow::Break => break,
                        ControlFlow::Continue => {
                            i += 1;
                            continue;
                        }
                        ControlFlow::None => {}
                    }
                    i += 1;
                }
            }
            Statement::Break { .. } => {
                return Ok(ControlFlow::Break);
            }
            Statement::Continue { .. } => {
                return Ok(ControlFlow::Continue);
            }
            Statement::ExpressionStatement { expression } => {
                self.evaluate_expression(expression)?;
            }
        }
        Ok(ControlFlow::None)
    }

    // ── Expressions ───────────────────────────────────────────────────────────

    pub fn evaluate_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<RuntimeValue, CompilerError> {
        match expr {
            Expression::IntegerLiteral { value, .. } => Ok(RuntimeValue::Integer(*value)),
            Expression::FloatLiteral { value, .. } => Ok(RuntimeValue::Float(*value)),
            Expression::StringLiteral { value, .. } => Ok(RuntimeValue::String(value.clone())),
            Expression::BooleanLiteral { value, .. } => Ok(RuntimeValue::Boolean(*value)),
            Expression::NullLiteral { .. } => Ok(RuntimeValue::Null),
            Expression::Identifier { name, line, column } => {
                self.environment.borrow().get(name, *line, *column)
            }
            Expression::Grouping { expression, .. } => self.evaluate_expression(expression),
            Expression::UnaryOp {
                operator,
                operand,
                line,
                column,
            } => {
                let val = self.evaluate_expression(operand)?;
                match operator {
                    UnaryOperator::Negate => match val {
                        RuntimeValue::Integer(v) => Ok(RuntimeValue::Integer(v.wrapping_neg())),
                        RuntimeValue::Float(v) => Ok(RuntimeValue::Float(-v)),
                        other => Err(CompilerError::TypeMismatch {
                            operation: "unary negation '-'".to_string(),
                            expected: "Integer or Float".to_string(),
                            found: other.type_name().to_string(),
                            line: *line,
                            column: *column,
                        }),
                    },
                    UnaryOperator::LogicalNot => match val {
                        RuntimeValue::Boolean(v) => Ok(RuntimeValue::Boolean(!v)),
                        other => Err(CompilerError::TypeMismatch {
                            operation: "logical negation '!'".to_string(),
                            expected: "Boolean".to_string(),
                            found: other.type_name().to_string(),
                            line: *line,
                            column: *column,
                        }),
                    },
                }
            }
            Expression::BinaryOp {
                operator,
                left,
                right,
                line,
                column,
            } => {
                // Short-circuiting logical operations need to be evaluated specially
                if *operator == BinaryOperator::And {
                    let left_val = self.evaluate_expression(left)?;
                    let left_bool = match left_val {
                        RuntimeValue::Boolean(b) => b,
                        other => {
                            return Err(CompilerError::TypeMismatch {
                                operation: "logical AND '&&'".to_string(),
                                expected: "Boolean".to_string(),
                                found: other.type_name().to_string(),
                                line: *line,
                                column: *column,
                            });
                        }
                    };
                    if !left_bool {
                        return Ok(RuntimeValue::Boolean(false));
                    }
                    let right_val = self.evaluate_expression(right)?;
                    match right_val {
                        RuntimeValue::Boolean(b) => return Ok(RuntimeValue::Boolean(b)),
                        other => {
                            return Err(CompilerError::TypeMismatch {
                                operation: "logical AND '&&'".to_string(),
                                expected: "Boolean".to_string(),
                                found: other.type_name().to_string(),
                                line: *line,
                                column: *column,
                            });
                        }
                    }
                }

                if *operator == BinaryOperator::Or {
                    let left_val = self.evaluate_expression(left)?;
                    let left_bool = match left_val {
                        RuntimeValue::Boolean(b) => b,
                        other => {
                            return Err(CompilerError::TypeMismatch {
                                operation: "logical OR '||'".to_string(),
                                expected: "Boolean".to_string(),
                                found: other.type_name().to_string(),
                                line: *line,
                                column: *column,
                            });
                        }
                    };
                    if left_bool {
                        return Ok(RuntimeValue::Boolean(true));
                    }
                    let right_val = self.evaluate_expression(right)?;
                    match right_val {
                        RuntimeValue::Boolean(b) => return Ok(RuntimeValue::Boolean(b)),
                        other => {
                            return Err(CompilerError::TypeMismatch {
                                operation: "logical OR '||'".to_string(),
                                expected: "Boolean".to_string(),
                                found: other.type_name().to_string(),
                                line: *line,
                                column: *column,
                            });
                        }
                    }
                }

                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;

                match operator {
                    BinaryOperator::Add => self.eval_add(left_val, right_val, *line, *column),
                    BinaryOperator::Subtract => {
                        self.eval_arithmetic(left_val, right_val, "-", *line, *column)
                    }
                    BinaryOperator::Multiply => {
                        self.eval_arithmetic(left_val, right_val, "*", *line, *column)
                    }
                    BinaryOperator::Divide => {
                        self.eval_division(left_val, right_val, "/", *line, *column)
                    }
                    BinaryOperator::Modulo => {
                        self.eval_division(left_val, right_val, "%", *line, *column)
                    }
                    BinaryOperator::Equal => Ok(RuntimeValue::Boolean(
                        self.eval_equality(&left_val, &right_val),
                    )),
                    BinaryOperator::NotEqual => Ok(RuntimeValue::Boolean(
                        !self.eval_equality(&left_val, &right_val),
                    )),
                    BinaryOperator::Less => {
                        self.eval_comparison(left_val, right_val, "<", *line, *column)
                    }
                    BinaryOperator::Greater => {
                        self.eval_comparison(left_val, right_val, ">", *line, *column)
                    }
                    BinaryOperator::LessEqual => {
                        self.eval_comparison(left_val, right_val, "<=", *line, *column)
                    }
                    BinaryOperator::GreaterEqual => {
                        self.eval_comparison(left_val, right_val, ">=", *line, *column)
                    }
                    BinaryOperator::And | BinaryOperator::Or => unreachable!(),
                }
            }
            Expression::Range {
                start,
                end,
                inclusive,
                line,
                column,
            } => {
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;

                let start_i64 = match start_val {
                    RuntimeValue::Integer(v) => v,
                    other => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "range start bound".to_string(),
                            expected: "Integer".to_string(),
                            found: other.type_name().to_string(),
                            line: *line,
                            column: *column,
                        });
                    }
                };

                let end_i64 = match end_val {
                    RuntimeValue::Integer(v) => v,
                    other => {
                        return Err(CompilerError::TypeMismatch {
                            operation: "range end bound".to_string(),
                            expected: "Integer".to_string(),
                            found: other.type_name().to_string(),
                            line: *line,
                            column: *column,
                        });
                    }
                };

                Ok(RuntimeValue::Range {
                    start: start_i64,
                    end: end_i64,
                    inclusive: *inclusive,
                })
            }
        }
    }

    // ── Helper Evaluators ─────────────────────────────────────────────────────

    fn eval_add(
        &self,
        left: RuntimeValue,
        right: RuntimeValue,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        match (left, right) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Integer(l.wrapping_add(r)))
            }
            (RuntimeValue::Float(l), RuntimeValue::Float(r)) => Ok(RuntimeValue::Float(l + r)),
            (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => {
                Ok(RuntimeValue::Float(l as f64 + r))
            }
            (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => {
                Ok(RuntimeValue::Float(l + r as f64))
            }
            (RuntimeValue::String(l), RuntimeValue::String(r)) => {
                Ok(RuntimeValue::String(format!("{l}{r}")))
            }
            (l, r) => Err(CompilerError::TypeMismatch {
                operation: "addition '+'".to_string(),
                expected: "numbers or strings".to_string(),
                found: format!("{} and {}", l.type_name(), r.type_name()),
                line,
                column,
            }),
        }
    }

    fn eval_arithmetic(
        &self,
        left: RuntimeValue,
        right: RuntimeValue,
        op: &str,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        match (left, right) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => match op {
                "-" => Ok(RuntimeValue::Integer(l.wrapping_sub(r))),
                "*" => Ok(RuntimeValue::Integer(l.wrapping_mul(r))),
                _ => unreachable!(),
            },
            (RuntimeValue::Float(l), RuntimeValue::Float(r)) => match op {
                "-" => Ok(RuntimeValue::Float(l - r)),
                "*" => Ok(RuntimeValue::Float(l * r)),
                _ => unreachable!(),
            },
            (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => match op {
                "-" => Ok(RuntimeValue::Float(l as f64 - r)),
                "*" => Ok(RuntimeValue::Float(l as f64 * r)),
                _ => unreachable!(),
            },
            (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => match op {
                "-" => Ok(RuntimeValue::Float(l - r as f64)),
                "*" => Ok(RuntimeValue::Float(l * r as f64)),
                _ => unreachable!(),
            },
            (l, r) => Err(CompilerError::TypeMismatch {
                operation: format!("arithmetic '{op}'"),
                expected: "numbers (Integer or Float)".to_string(),
                found: format!("{} and {}", l.type_name(), r.type_name()),
                line,
                column,
            }),
        }
    }

    fn eval_division(
        &self,
        left: RuntimeValue,
        right: RuntimeValue,
        op: &str,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        // Coerce to common type
        match (left, right) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => {
                if r == 0 {
                    return Err(CompilerError::DivisionByZero { line, column });
                }
                match op {
                    "/" => Ok(RuntimeValue::Integer(l.wrapping_div(r))),
                    "%" => Ok(RuntimeValue::Integer(l.wrapping_rem(r))),
                    _ => unreachable!(),
                }
            }
            (RuntimeValue::Float(l), RuntimeValue::Float(r)) => {
                if r == 0.0 {
                    return Err(CompilerError::DivisionByZero { line, column });
                }
                match op {
                    "/" => Ok(RuntimeValue::Float(l / r)),
                    "%" => Ok(RuntimeValue::Float(l % r)),
                    _ => unreachable!(),
                }
            }
            (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => {
                if r == 0.0 {
                    return Err(CompilerError::DivisionByZero { line, column });
                }
                let lf = l as f64;
                match op {
                    "/" => Ok(RuntimeValue::Float(lf / r)),
                    "%" => Ok(RuntimeValue::Float(lf % r)),
                    _ => unreachable!(),
                }
            }
            (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => {
                if r == 0 {
                    return Err(CompilerError::DivisionByZero { line, column });
                }
                let rf = r as f64;
                match op {
                    "/" => Ok(RuntimeValue::Float(l / rf)),
                    "%" => Ok(RuntimeValue::Float(l % rf)),
                    _ => unreachable!(),
                }
            }
            (l, r) => Err(CompilerError::TypeMismatch {
                operation: format!("division/modulo '{op}'"),
                expected: "numbers (Integer or Float)".to_string(),
                found: format!("{} and {}", l.type_name(), r.type_name()),
                line,
                column,
            }),
        }
    }

    fn eval_equality(&self, left: &RuntimeValue, right: &RuntimeValue) -> bool {
        match (left, right) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => l == r,
            (RuntimeValue::Float(l), RuntimeValue::Float(r)) => l == r,
            (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => *l as f64 == *r,
            (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => *l == *r as f64,
            (RuntimeValue::String(l), RuntimeValue::String(r)) => l == r,
            (RuntimeValue::Boolean(l), RuntimeValue::Boolean(r)) => l == r,
            (RuntimeValue::Null, RuntimeValue::Null) => true,
            _ => false,
        }
    }

    fn eval_comparison(
        &self,
        left: RuntimeValue,
        right: RuntimeValue,
        op: &str,
        line: usize,
        column: usize,
    ) -> Result<RuntimeValue, CompilerError> {
        match (left, right) {
            (RuntimeValue::Integer(l), RuntimeValue::Integer(r)) => match op {
                "<" => Ok(RuntimeValue::Boolean(l < r)),
                ">" => Ok(RuntimeValue::Boolean(l > r)),
                "<=" => Ok(RuntimeValue::Boolean(l <= r)),
                ">=" => Ok(RuntimeValue::Boolean(l >= r)),
                _ => unreachable!(),
            },
            (RuntimeValue::Float(l), RuntimeValue::Float(r)) => match op {
                "<" => Ok(RuntimeValue::Boolean(l < r)),
                ">" => Ok(RuntimeValue::Boolean(l > r)),
                "<=" => Ok(RuntimeValue::Boolean(l <= r)),
                ">=" => Ok(RuntimeValue::Boolean(l >= r)),
                _ => unreachable!(),
            },
            (RuntimeValue::Integer(l), RuntimeValue::Float(r)) => {
                let lf = l as f64;
                match op {
                    "<" => Ok(RuntimeValue::Boolean(lf < r)),
                    ">" => Ok(RuntimeValue::Boolean(lf > r)),
                    "<=" => Ok(RuntimeValue::Boolean(lf <= r)),
                    ">=" => Ok(RuntimeValue::Boolean(lf >= r)),
                    _ => unreachable!(),
                }
            }
            (RuntimeValue::Float(l), RuntimeValue::Integer(r)) => {
                let rf = r as f64;
                match op {
                    "<" => Ok(RuntimeValue::Boolean(l < rf)),
                    ">" => Ok(RuntimeValue::Boolean(l > rf)),
                    "<=" => Ok(RuntimeValue::Boolean(l <= rf)),
                    ">=" => Ok(RuntimeValue::Boolean(l >= rf)),
                    _ => unreachable!(),
                }
            }
            (RuntimeValue::String(l), RuntimeValue::String(r)) => match op {
                "<" => Ok(RuntimeValue::Boolean(l < r)),
                ">" => Ok(RuntimeValue::Boolean(l > r)),
                "<=" => Ok(RuntimeValue::Boolean(l <= r)),
                ">=" => Ok(RuntimeValue::Boolean(l >= r)),
                _ => unreachable!(),
            },
            (l, r) => Err(CompilerError::TypeMismatch {
                operation: format!("comparison '{op}'"),
                expected: "numbers or strings".to_string(),
                found: format!("{} and {}", l.type_name(), r.type_name()),
                line,
                column,
            }),
        }
    }
}
