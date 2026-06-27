pub mod handlers;
pub mod protocol;

#[cfg(test)]
pub mod tests;

use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use serde_json::Value;

use protocol::{
    Request, Response, Notification, InitializeResult, ServerCapabilities,
    PublishDiagnosticsParams, HoverParams, DefinitionParams
};

/// Runs the LSP server loop over stdin/stdout.
pub fn run() -> Result<(), String> {
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    let mut documents: HashMap<String, String> = HashMap::new();

    loop {
        let mut content_length = 0;
        
        // Read headers
        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                // EOF
                return Ok(());
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                // End of headers
                break;
            }

            if line.to_lowercase().starts_with("content-length:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    content_length = parts[1].trim().parse::<usize>().unwrap_or(0);
                }
            }
        }

        if content_length == 0 {
            continue;
        }

        // Read body
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).map_err(|e| e.to_string())?;

        let body_str = String::from_utf8(body).map_err(|e| e.to_string())?;

        // Parse and handle message
        if let Ok(req) = serde_json::from_str::<Request>(&body_str) {
            handle_request(&req, &mut stdout, &mut documents)?;
            // If the client requested shutdown, we exit the loop.
            if req.method == "shutdown" {
                break;
            }
        } else if let Ok(notif) = serde_json::from_str::<Notification>(&body_str) {
            handle_notification(&notif, &mut stdout, &mut documents)?;
        }
    }

    Ok(())
}

fn send_response<W: Write>(writer: &mut W, resp: Response) -> Result<(), String> {
    let body = serde_json::to_string(&resp).map_err(|e| e.to_string())?;
    write_message(writer, &body)
}

fn send_notification<W: Write>(writer: &mut W, notif: Notification) -> Result<(), String> {
    let body = serde_json::to_string(&notif).map_err(|e| e.to_string())?;
    write_message(writer, &body)
}

fn write_message<W: Write>(writer: &mut W, body: &str) -> Result<(), String> {
    let msg = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    writer.write_all(msg.as_bytes()).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn handle_request<W: Write>(
    req: &Request,
    writer: &mut W,
    documents: &mut HashMap<String, String>,
) -> Result<(), String> {
    let mut response = Response {
        jsonrpc: "2.0".to_string(),
        id: req.id.clone(),
        result: None,
        error: None,
    };

    match req.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                capabilities: ServerCapabilities {
                    text_document_sync: 1, // Full document sync
                    hover_provider: true,
                    definition_provider: true,
                },
            };
            response.result = Some(serde_json::to_value(result).unwrap());
        }
        "textDocument/hover" => {
            if let Some(params_val) = &req.params {
                if let Ok(params) = serde_json::from_value::<HoverParams>(params_val.clone()) {
                    if let Some(doc_text) = documents.get(&params.text_document.uri) {
                        if let Some(hover) = handlers::handle_hover(doc_text, params.position.line, params.position.character) {
                            response.result = Some(serde_json::to_value(hover).unwrap());
                        } else {
                            response.result = Some(Value::Null);
                        }
                    }
                }
            }
        }
        "textDocument/definition" => {
            if let Some(params_val) = &req.params {
                if let Ok(params) = serde_json::from_value::<DefinitionParams>(params_val.clone()) {
                    if let Some(doc_text) = documents.get(&params.text_document.uri) {
                        if let Some(loc) = handlers::handle_definition(&params.text_document.uri, doc_text, params.position.line, params.position.character) {
                            response.result = Some(serde_json::to_value(loc).unwrap());
                        } else {
                            response.result = Some(Value::Null);
                        }
                    }
                }
            }
        }
        "shutdown" => {
            response.result = Some(Value::Null);
        }
        _ => {
            response.error = Some(protocol::ResponseError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
                data: None,
            });
        }
    }

    send_response(writer, response)
}

fn handle_notification<W: Write>(
    notif: &Notification,
    writer: &mut W,
    documents: &mut HashMap<String, String>,
) -> Result<(), String> {
    match notif.method.as_str() {
        "textDocument/didOpen" => {
            if let Some(params_val) = &notif.params {
                if let Ok(params) = serde_json::from_value::<protocol::DidOpenTextDocumentParams>(params_val.clone()) {
                    documents.insert(params.text_document.uri.clone(), params.text_document.text.clone());
                    publish_diagnostics(&params.text_document.uri, &params.text_document.text, writer)?;
                }
            }
        }
        "textDocument/didChange" => {
            if let Some(params_val) = &notif.params {
                if let Ok(params) = serde_json::from_value::<protocol::DidChangeTextDocumentParams>(params_val.clone()) {
                    if let Some(change) = params.content_changes.first() {
                        documents.insert(params.text_document.uri.clone(), change.text.clone());
                        publish_diagnostics(&params.text_document.uri, &change.text, writer)?;
                    }
                }
            }
        }
        "textDocument/didClose" => {
            if let Some(params_val) = &notif.params {
                if let Ok(params) = serde_json::from_value::<protocol::DefinitionParams>(params_val.clone()) {
                    documents.remove(&params.text_document.uri);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn publish_diagnostics<W: Write>(uri: &str, text: &str, writer: &mut W) -> Result<(), String> {
    let diagnostics = handlers::compile_and_get_diagnostics(text);
    let params = PublishDiagnosticsParams {
        uri: uri.to_string(),
        diagnostics,
    };
    let notif = Notification {
        jsonrpc: "2.0".to_string(),
        method: "textDocument/publishDiagnostics".to_string(),
        params: Some(serde_json::to_value(params).unwrap()),
    };
    send_notification(writer, notif)
}
