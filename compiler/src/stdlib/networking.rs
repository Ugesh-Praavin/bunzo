//! Networking module: TCP client, TCP server, UDP socket, UDP send, UDP receive using Rust's standard library.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{Mutex, OnceLock};

use crate::diagnostics::CompilerError;
use crate::runtime::value::RuntimeValue;
use super::{make_builtin, module_map};

struct NetRegistry {
    next_id: usize,
    listeners: HashMap<usize, TcpListener>,
    streams: HashMap<usize, TcpStream>,
    udp_sockets: HashMap<usize, UdpSocket>,
}

fn get_registry() -> &'static Mutex<NetRegistry> {
    static REGISTRY: OnceLock<Mutex<NetRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(NetRegistry {
        next_id: 1,
        listeners: HashMap::new(),
        streams: HashMap::new(),
        udp_sockets: HashMap::new(),
    }))
}

pub fn build() -> RuntimeValue {
    let mut map = HashMap::new();
    map.insert(
        "tcp_listen".to_string(),
        make_builtin("networking.tcp_listen", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "networking.tcp_listen".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::String(addr) = &args[0] {
                match TcpListener::bind(addr) {
                    Ok(listener) => {
                        let mut reg = get_registry().lock().unwrap();
                        let id = reg.next_id;
                        reg.next_id += 1;
                        reg.listeners.insert(id, listener);
                        return Ok(RuntimeValue::Integer(id as i64));
                    }
                    Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to bind TCP listener to {addr}: {e}"), line: l, column: c })
                }
            }
            Err(CompilerError::TypeMismatch { operation: "networking.tcp_listen".into(), expected: "String".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "tcp_accept".to_string(),
        make_builtin("networking.tcp_accept", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "networking.tcp_accept".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::Integer(id) = &args[0] {
                let id = *id as usize;
                let listener = {
                    let reg = get_registry().lock().unwrap();
                    reg.listeners.get(&id).map(|l| l.try_clone().unwrap())
                };
                if let Some(listener) = listener {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let mut reg = get_registry().lock().unwrap();
                            let client_id = reg.next_id;
                            reg.next_id += 1;
                            reg.streams.insert(client_id, stream);
                            return Ok(RuntimeValue::Integer(client_id as i64));
                        }
                        Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to accept TCP connection: {e}"), line: l, column: c })
                    }
                }
                return Err(CompilerError::RuntimeException { message: format!("invalid TCP listener handle: {id}"), line: l, column: c });
            }
            Err(CompilerError::TypeMismatch { operation: "networking.tcp_accept".into(), expected: "Integer".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "tcp_connect".to_string(),
        make_builtin("networking.tcp_connect", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "networking.tcp_connect".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::String(addr) = &args[0] {
                match TcpStream::connect(addr) {
                    Ok(stream) => {
                        let mut reg = get_registry().lock().unwrap();
                        let id = reg.next_id;
                        reg.next_id += 1;
                        reg.streams.insert(id, stream);
                        return Ok(RuntimeValue::Integer(id as i64));
                    }
                    Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to connect TCP to {addr}: {e}"), line: l, column: c })
                }
            }
            Err(CompilerError::TypeMismatch { operation: "networking.tcp_connect".into(), expected: "String".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "tcp_send".to_string(),
        make_builtin("networking.tcp_send", |args, l, c| {
            if args.len() != 2 {
                return Err(CompilerError::ArityMismatch { name: "networking.tcp_send".into(), expected: 2, found: args.len(), line: l, column: c });
            }
            if let (RuntimeValue::Integer(id), RuntimeValue::String(data)) = (&args[0], &args[1]) {
                let id = *id as usize;
                let stream = {
                    let reg = get_registry().lock().unwrap();
                    reg.streams.get(&id).map(|s| s.try_clone().unwrap())
                };
                if let Some(mut stream) = stream {
                    match stream.write_all(data.as_bytes()) {
                        Ok(_) => return Ok(RuntimeValue::Null),
                        Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to send TCP data: {e}"), line: l, column: c })
                    }
                }
                return Err(CompilerError::RuntimeException { message: format!("invalid TCP stream handle: {id}"), line: l, column: c });
            }
            Err(CompilerError::TypeMismatch { operation: "networking.tcp_send".into(), expected: "Integer and String".into(), found: format!("{}, {}", args[0].type_name(), args[1].type_name()), line: l, column: c })
        }),
    );
    map.insert(
        "tcp_recv".to_string(),
        make_builtin("networking.tcp_recv", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "networking.tcp_recv".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::Integer(id) = &args[0] {
                let id = *id as usize;
                let stream = {
                    let reg = get_registry().lock().unwrap();
                    reg.streams.get(&id).map(|s| s.try_clone().unwrap())
                };
                if let Some(mut stream) = stream {
                    let mut buf = [0; 1024];
                    match stream.read(&mut buf) {
                        Ok(n) => {
                            let data = String::from_utf8_lossy(&buf[..n]).to_string();
                            return Ok(RuntimeValue::String(data));
                        }
                        Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to receive TCP data: {e}"), line: l, column: c })
                    }
                }
                return Err(CompilerError::RuntimeException { message: format!("invalid TCP stream handle: {id}"), line: l, column: c });
            }
            Err(CompilerError::TypeMismatch { operation: "networking.tcp_recv".into(), expected: "Integer".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "udp_bind".to_string(),
        make_builtin("networking.udp_bind", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "networking.udp_bind".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::String(addr) = &args[0] {
                match UdpSocket::bind(addr) {
                    Ok(sock) => {
                        let mut reg = get_registry().lock().unwrap();
                        let id = reg.next_id;
                        reg.next_id += 1;
                        reg.udp_sockets.insert(id, sock);
                        return Ok(RuntimeValue::Integer(id as i64));
                    }
                    Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to bind UDP socket to {addr}: {e}"), line: l, column: c })
                }
            }
            Err(CompilerError::TypeMismatch { operation: "networking.udp_bind".into(), expected: "String".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    map.insert(
        "udp_send".to_string(),
        make_builtin("networking.udp_send", |args, l, c| {
            if args.len() != 3 {
                return Err(CompilerError::ArityMismatch { name: "networking.udp_send".into(), expected: 3, found: args.len(), line: l, column: c });
            }
            if let (RuntimeValue::Integer(id), RuntimeValue::String(addr), RuntimeValue::String(data)) = (&args[0], &args[1], &args[2]) {
                let id = *id as usize;
                let sock = {
                    let reg = get_registry().lock().unwrap();
                    reg.udp_sockets.get(&id).map(|s| s.try_clone().unwrap())
                };
                if let Some(sock) = sock {
                    match sock.send_to(data.as_bytes(), addr) {
                        Ok(_) => return Ok(RuntimeValue::Null),
                        Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to send UDP packet to {addr}: {e}"), line: l, column: c })
                    }
                }
                return Err(CompilerError::RuntimeException { message: format!("invalid UDP socket handle: {id}"), line: l, column: c });
            }
            Err(CompilerError::TypeMismatch { operation: "networking.udp_send".into(), expected: "Integer, String, and String".into(), found: format!("{}, {}, {}", args[0].type_name(), args[1].type_name(), args[2].type_name()), line: l, column: c })
        }),
    );
    map.insert(
        "udp_recv".to_string(),
        make_builtin("networking.udp_recv", |args, l, c| {
            if args.len() != 1 {
                return Err(CompilerError::ArityMismatch { name: "networking.udp_recv".into(), expected: 1, found: args.len(), line: l, column: c });
            }
            if let RuntimeValue::Integer(id) = &args[0] {
                let id = *id as usize;
                let sock = {
                    let reg = get_registry().lock().unwrap();
                    reg.udp_sockets.get(&id).map(|s| s.try_clone().unwrap())
                };
                if let Some(sock) = sock {
                    let mut buf = [0; 1024];
                    match sock.recv_from(&mut buf) {
                        Ok((n, _)) => {
                            let data = String::from_utf8_lossy(&buf[..n]).to_string();
                            return Ok(RuntimeValue::String(data));
                        }
                        Err(e) => return Err(CompilerError::RuntimeException { message: format!("failed to receive UDP packet: {e}"), line: l, column: c })
                    }
                }
                return Err(CompilerError::RuntimeException { message: format!("invalid UDP socket handle: {id}"), line: l, column: c });
            }
            Err(CompilerError::TypeMismatch { operation: "networking.udp_recv".into(), expected: "Integer".into(), found: args[0].type_name().to_string(), line: l, column: c })
        }),
    );
    module_map(map)
}
