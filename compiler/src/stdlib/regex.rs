//! Regex module: match, search, find, find_all, replace, split, is_match.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{make_builtin, module_map};
use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

// A simple recursive regex matching engine for standard library-like pattern matching.
// Supports: `.` (any char), `*` (zero or more), `^` (start), `$` (end).
fn match_here(re: &[char], text: &[char]) -> bool {
    if re.is_empty() {
        return true;
    }
    if re[0] == '$' && re.len() == 1 {
        return text.is_empty();
    }
    if re.len() >= 2 && re[1] == '*' {
        return match_star(re[0], &re[2..], text);
    }
    if !text.is_empty() && (re[0] == '.' || re[0] == text[0]) {
        return match_here(&re[1..], &text[1..]);
    }
    false
}

fn match_star(c: char, re: &[char], text: &[char]) -> bool {
    let mut i = 0;
    loop {
        if match_here(re, &text[i..]) {
            return true;
        }
        if i < text.len() && (c == '.' || text[i] == c) {
            i += 1;
        } else {
            break;
        }
    }
    false
}

fn is_match(pattern: &str, text: &str) -> bool {
    let re: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    if re.first() == Some(&'^') {
        return match_here(&re[1..], &txt);
    }
    let mut i = 0;
    loop {
        if match_here(&re, &txt[i..]) {
            return true;
        }
        if i >= txt.len() {
            break;
        }
        i += 1;
    }
    false
}

// Find matched text substring.
fn find_match(pattern: &str, text: &str) -> Option<(usize, usize)> {
    let re: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();

    // We want to find the first matching slice.
    // Try matching from every starting index.
    for i in 0..=txt.len() {
        // If pattern starts with ^, it can only match at index 0.
        if re.first() == Some(&'^') && i > 0 {
            break;
        }
        let re_slice = if re.first() == Some(&'^') {
            &re[1..]
        } else {
            &re
        };

        // Find how many characters match. We search for the shortest/longest match.
        // Let's do greedy matching (longest possible match).
        let mut len = txt.len() - i;
        while len > 0 {
            if match_here(re_slice, &txt[i..(i + len)]) {
                return Some((i, i + len));
            }
            len -= 1;
        }
        // Check for 0-length match if applicable
        if match_here(re_slice, &txt[i..i]) {
            return Some((i, i));
        }
    }
    None
}

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "is_match".to_string(),
        make_builtin("regex.is_match", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "regex.is_match".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(pat), RuntimeValue::String(txt)) = (&args[0], &args[1]) {
                return Ok(RuntimeValue::Boolean(is_match(pat, txt)));
            }
            Err(CompilerError::TypeMismatch {
                operation: "regex.is_match".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "match".to_string(),
        make_builtin("regex.match", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "regex.match".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(pat), RuntimeValue::String(txt)) = (&args[0], &args[1]) {
                if let Some((start, end)) = find_match(pat, txt) {
                    return Ok(RuntimeValue::String(txt[start..end].to_string()));
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch {
                operation: "regex.match".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "find".to_string(),
        make_builtin("regex.find", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "regex.find".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(pat), RuntimeValue::String(txt)) = (&args[0], &args[1]) {
                if let Some((start, end)) = find_match(pat, txt) {
                    return Ok(RuntimeValue::String(txt[start..end].to_string()));
                }
                return Ok(RuntimeValue::Null);
            }
            Err(CompilerError::TypeMismatch {
                operation: "regex.find".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "search".to_string(),
        make_builtin("regex.search", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "regex.search".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(pat), RuntimeValue::String(txt)) = (&args[0], &args[1]) {
                if let Some((start, _)) = find_match(pat, txt) {
                    return Ok(RuntimeValue::Integer(start as i64));
                }
                return Ok(RuntimeValue::Integer(-1));
            }
            Err(CompilerError::TypeMismatch {
                operation: "regex.search".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "find_all".to_string(),
        make_builtin("regex.find_all", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "regex.find_all".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(pat), RuntimeValue::String(txt)) = (&args[0], &args[1]) {
                let mut matches = Vec::new();
                let mut current_txt = txt.as_str();
                while let Some((start, end)) = find_match(pat, current_txt) {
                    let match_str = &current_txt[start..end];
                    matches.push(RuntimeValue::String(match_str.to_string()));
                    let next_start = if end == 0 { 1 } else { end };
                    if next_start >= current_txt.len() {
                        break;
                    }
                    current_txt = &current_txt[next_start..];
                }
                return Ok(RuntimeValue::Array(Rc::new(RefCell::new(matches))));
            }
            Err(CompilerError::TypeMismatch {
                operation: "regex.find_all".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "replace".to_string(),
        make_builtin("regex.replace", |args, l, c| {
            if args.len() != 3 {
                return Err(CompilerError::ArityMismatch {
                    name: "regex.replace".into(),
                    expected: 3,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (
                RuntimeValue::String(pat),
                RuntimeValue::String(txt),
                RuntimeValue::String(rep),
            ) = (&args[0], &args[1], &args[2])
            {
                if let Some((start, end)) = find_match(pat, txt) {
                    let mut result = txt[..start].to_string();
                    result.push_str(rep);
                    result.push_str(&txt[end..]);
                    return Ok(RuntimeValue::String(result));
                }
                return Ok(RuntimeValue::String(txt.clone()));
            }
            Err(CompilerError::TypeMismatch {
                operation: "regex.replace".into(),
                expected: "String, String, and String".into(),
                found: format!(
                    "{}, {}, {}",
                    args[0].type_name(),
                    args[1].type_name(),
                    args[2].type_name()
                ),
                line: l,
                column: c,
            })
        }),
    );
    map.insert(
        "split".to_string(),
        make_builtin("regex.split", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch {
                    name: "regex.split".into(),
                    expected: 2,
                    found: args.len(),
                    line: l,
                    column: c,
                });
            }
            if let (RuntimeValue::String(pat), RuntimeValue::String(txt)) = (&args[0], &args[1]) {
                let mut parts = Vec::new();
                let mut current_txt = txt.as_str();
                while let Some((start, end)) = find_match(pat, current_txt) {
                    parts.push(RuntimeValue::String(current_txt[..start].to_string()));
                    let next_start = if end == 0 { 1 } else { end };
                    if next_start >= current_txt.len() {
                        current_txt = "";
                        break;
                    }
                    current_txt = &current_txt[next_start..];
                }
                if !current_txt.is_empty() || parts.is_empty() {
                    parts.push(RuntimeValue::String(current_txt.to_string()));
                }
                return Ok(RuntimeValue::Array(Rc::new(RefCell::new(parts))));
            }
            Err(CompilerError::TypeMismatch {
                operation: "regex.split".into(),
                expected: "String and String".into(),
                found: format!("{}, {}", args[0].type_name(), args[1].type_name()),
                line: l,
                column: c,
            })
        }),
    );
    module_map(map)
}
