//! HTTP client module: `import http` — GET, POST, PUT, DELETE, PATCH.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;

use super::{make_builtin, module_map};

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert("get".to_string(), make_builtin("http.get", http_get));
    map.insert("post".to_string(), make_builtin("http.post", http_post));
    map.insert("put".to_string(), make_builtin("http.put", http_put));
    map.insert("delete".to_string(), make_builtin("http.delete", http_delete));
    map.insert("patch".to_string(), make_builtin("http.patch", http_patch));
    module_map(map)
}

struct HttpUrlParts<'a> {
    host: &'a str,
    port: u16,
    path: &'a str,
    is_https: bool,
}

fn parse_http_url(url: &str) -> Result<HttpUrlParts<'_>, String> {
    let is_https = url.starts_with("https://");
    let stripped = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let slash = stripped.find('/').unwrap_or(stripped.len());
    let host_port = &stripped[..slash];
    let path = if slash < stripped.len() {
        &stripped[slash..]
    } else {
        "/"
    };
    let (host, port) = if let Some(colon) = host_port.find(':') {
        let p: u16 = host_port[colon + 1..]
            .parse()
            .map_err(|_| format!("invalid port in URL: {url}"))?;
        (&host_port[..colon], p)
    } else if is_https {
        (host_port, 443)
    } else {
        (host_port, 80)
    };
    Ok(HttpUrlParts {
        host,
        port,
        path,
        is_https,
    })
}

fn http_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    content_type: Option<&str>,
) -> Result<String, String> {
    if parse_http_url(url)?.is_https {
        return Err("https URLs are not supported yet; use http://".to_string());
    }

    let parts = parse_http_url(url)?;
    let addr = format!("{}:{}", parts.host, parts.port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();

    let mut request = format!(
        "{method} {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n",
        parts.path, parts.host
    );
    if let Some(b) = body {
        let ct = content_type.unwrap_or("application/json");
        request.push_str(&format!("Content-Type: {ct}\r\n"));
        request.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    request.push_str("\r\n");
    if let Some(b) = body {
        request.push_str(b);
    }

    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| e.to_string())?;

    if let Some(pos) = response.find("\r\n\r\n") {
        Ok(response[pos + 4..].to_string())
    } else {
        Ok(response)
    }
}

fn require_url(
    args: &[RuntimeValue],
    op: &str,
    line: usize,
    column: usize,
) -> Result<String, CompilerError> {
    if args.is_empty() {
        return Err(CompilerError::ArityMismatch {
            name: op.into(),
            expected: 1,
            found: 0,
            line,
            column,
        });
    }
    match &args[0] {
        RuntimeValue::String(s) => Ok(s.clone()),
        other => Err(CompilerError::TypeMismatch {
            operation: format!("{op}()"),
            expected: "String".into(),
            found: other.type_name().to_string(),
            line,
            column,
        }),
    }
}

fn require_url_body(
    args: &[RuntimeValue],
    op: &str,
    line: usize,
    column: usize,
) -> Result<(String, String, String), CompilerError> {
    if args.len() < 2 {
        return Err(CompilerError::ArityMismatch {
            name: op.into(),
            expected: 2,
            found: args.len(),
            line,
            column,
        });
    }
    let url = require_url(args, op, line, column)?;
    let body = format!("{}", args[1]);
    let content_type = if args.len() >= 3 {
        match &args[2] {
            RuntimeValue::String(s) => s.clone(),
            other => {
                return Err(CompilerError::TypeMismatch {
                    operation: format!("{op}()"),
                    expected: "String".into(),
                    found: other.type_name().to_string(),
                    line,
                    column,
                });
            }
        }
    } else {
        "application/json".to_string()
    };
    Ok((url, body, content_type))
}

fn run_request(
    method: &str,
    args: Vec<RuntimeValue>,
    op: &str,
    line: usize,
    column: usize,
    with_body: bool,
) -> Result<RuntimeValue, CompilerError> {
    let result = if with_body {
        let (url, body, content_type) = require_url_body(&args, op, line, column)?;
        http_request(method, &url, Some(&body), Some(&content_type))
    } else {
        let url = require_url(&args, op, line, column)?;
        http_request(method, &url, None, None)
    };
    match result {
        Ok(body) => Ok(RuntimeValue::String(body)),
        Err(e) => Err(CompilerError::RuntimeException {
            message: e,
            line,
            column,
        }),
    }
}

fn http_get(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    run_request("GET", args, "http.get", line, column, false)
}

fn http_post(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    run_request("POST", args, "http.post", line, column, true)
}

fn http_put(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    run_request("PUT", args, "http.put", line, column, true)
}

fn http_delete(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    run_request("DELETE", args, "http.delete", line, column, false)
}

fn http_patch(args: Vec<RuntimeValue>, line: usize, column: usize) -> Result<RuntimeValue, CompilerError> {
    run_request("PATCH", args, "http.patch", line, column, true)
}
