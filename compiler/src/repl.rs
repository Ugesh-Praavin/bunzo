use crate::lexer::tokenize;
use crate::parser::parse;
use crate::runtime::{Interpreter, value::RuntimeValue};
use crate::semantic::analyze;
use crate::typechecker::check as typecheck;
use std::io::{self, Write};

/// Starts the Bunzo interactive REPL.
pub fn run() -> Result<(), String> {
    println!("Bunzo Interactive REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' or press Ctrl+C to quit.");

    let mut interpreter = Interpreter::new(io::stdout());
    let mut history = String::new();

    loop {
        print!("bunzo> ");
        if let Err(e) = io::stdout().flush() {
            return Err(format!("Failed to flush stdout: {}", e));
        }

        let mut line = String::new();
        let bytes_read = match io::stdin().read_line(&mut line) {
            Ok(b) => b,
            Err(e) => return Err(format!("Failed to read stdin: {}", e)),
        };

        if bytes_read == 0 {
            // EOF (Ctrl+D)
            println!();
            break;
        }

        let trimmed = line.trim();
        if trimmed == "exit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        // Concatenate with history for parsing and static analysis
        let current_run = if history.is_empty() {
            trimmed.to_string()
        } else {
            format!("{}\n{}", history, trimmed)
        };

        // 1. Tokenize
        let tokens = match tokenize(&current_run) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[Lexer Error] {}", e);
                continue;
            }
        };

        // 2. Parse
        let program = match parse(tokens) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[Parser Error] {}", e);
                continue;
            }
        };

        // 3. Semantic Analysis
        if let Err(e) = analyze(&program) {
            eprintln!("[Semantic Error] {}", e);
            continue;
        }

        // 4. Type Checking
        if let Err(e) = typecheck(&program) {
            eprintln!("[Type Error] {}", e);
            continue;
        }

        // 5. Evaluate *only* the new statements
        let history_statement_count = if history.is_empty() {
            0
        } else {
            match tokenize(&history).and_then(parse) {
                Ok(p) => p.statements.len(),
                Err(_) => 0,
            }
        };

        let new_statements = &program.statements[history_statement_count..];

        let mut failed = false;
        for stmt in new_statements {
            match interpreter.interpret_statement(stmt) {
                Ok(Some(val)) => {
                    if val != RuntimeValue::Null {
                        println!("{}", val);
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("[Runtime Error] {}", e);
                    failed = true;
                    break;
                }
            }
        }

        // Save history if execution succeeded
        if !failed {
            if history.is_empty() {
                history = trimmed.to_string();
            } else {
                history.push_str("\n");
                history.push_str(trimmed);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Interpreter;

    #[test]
    fn test_repl_eval_variable_persistence() {
        let mut interpreter = Interpreter::new(Vec::new());
        let mut history = String::new();

        // Helper to run a line
        let mut run_line = |line: &str| -> Result<Option<String>, String> {
            let trimmed = line.trim();
            let current_run = if history.is_empty() {
                trimmed.to_string()
            } else {
                format!("{}\n{}", history, trimmed)
            };

            let tokens = tokenize(&current_run).map_err(|e| format!("{:?}", e))?;
            let program = parse(tokens).map_err(|e| format!("{:?}", e))?;
            analyze(&program).map_err(|e| format!("{:?}", e))?;
            typecheck(&program).map_err(|e| format!("{:?}", e))?;

            let history_statement_count = if history.is_empty() {
                0
            } else {
                let hist_tokens = tokenize(&history).unwrap();
                let hist_prog = parse(hist_tokens).unwrap();
                hist_prog.statements.len()
            };

            let new_statements = &program.statements[history_statement_count..];
            let mut last_val = None;
            for stmt in new_statements {
                let res = interpreter
                    .interpret_statement(stmt)
                    .map_err(|e| format!("{:?}", e))?;
                if let Some(val) = res {
                    if val != RuntimeValue::Null {
                        last_val = Some(format!("{}", val));
                    }
                }
            }

            if history.is_empty() {
                history = trimmed.to_string();
            } else {
                history.push_str("\n");
                history.push_str(trimmed);
            }

            Ok(last_val)
        };

        // Line 1: Define x
        assert_eq!(run_line("var x = 42").unwrap(), None);
        // Line 2: Print x (via bare expression)
        assert_eq!(run_line("x").unwrap(), Some("42".to_string()));
        // Line 3: Mutate x
        assert_eq!(run_line("x = x + 1").unwrap(), None);
        // Line 4: Print x again
        assert_eq!(run_line("x").unwrap(), Some("43".to_string()));
    }
}
