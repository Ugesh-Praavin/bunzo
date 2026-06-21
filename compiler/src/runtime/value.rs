use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::ast::Statement;
use super::environment::Environment;

/// A user-defined Bunzo function, as represented at runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct BzFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub closure: Rc<RefCell<Environment>>,
}

/// A user-defined Bunzo class, as represented at runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct BzClass {
    pub name: String,
    pub fields: Vec<String>,
    pub methods: HashMap<String, Rc<BzFunction>>,
}

/// A runtime value in the Bunzo interpreter.
#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    /// Sentinel: a value that has been moved away.
    Moved,
    /// A user-defined error string (from `?` propagation).
    Error(std::string::String),
    Function(Rc<BzFunction>),
    Struct {
        name: std::string::String,
        fields: HashMap<std::string::String, RuntimeValue>,
    },
    Class(Rc<BzClass>),
    Object {
        class_name: std::string::String,
        fields: Rc<RefCell<HashMap<std::string::String, RuntimeValue>>>,
        methods: HashMap<std::string::String, Rc<BzFunction>>,
    },
    BoundMethod {
        receiver: Rc<RuntimeValue>,
        method: Rc<BzFunction>,
    },
    Builtin {
        name: std::string::String,
        func: fn(Vec<RuntimeValue>, usize, usize) -> Result<RuntimeValue, crate::diagnostics::CompilerError>,
    },
    /// A dynamic array.
    Array(Rc<RefCell<Vec<RuntimeValue>>>),
    /// A string-keyed map (also used for modules).
    Map(Rc<RefCell<HashMap<std::string::String, RuntimeValue>>>),
    /// An enum variant value.
    EnumVariant {
        enum_name: std::string::String,
        variant: std::string::String,
        payload: Option<Rc<RuntimeValue>>,
    },
    /// A concurrent channel (thread-safe).
    Channel(Arc<Mutex<std::sync::mpsc::Receiver<RuntimeValue>>>),
    /// A channel sender half.
    ChannelSender(Arc<Mutex<std::sync::mpsc::Sender<RuntimeValue>>>),
}

impl RuntimeValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Integer(_)      => "Integer",
            RuntimeValue::Float(_)        => "Float",
            RuntimeValue::String(_)       => "String",
            RuntimeValue::Boolean(_)      => "Boolean",
            RuntimeValue::Null            => "Null",
            RuntimeValue::Moved           => "Moved",
            RuntimeValue::Error(_)        => "Error",
            RuntimeValue::Function(_)     => "Function",
            RuntimeValue::Struct { .. }   => "Struct",
            RuntimeValue::Class(_)        => "Class",
            RuntimeValue::Object { .. }   => "Object",
            RuntimeValue::BoundMethod { .. } => "BoundMethod",
            RuntimeValue::Builtin { .. }  => "Builtin",
            RuntimeValue::Array(_)        => "Array",
            RuntimeValue::Map(_)          => "Map",
            RuntimeValue::EnumVariant { .. } => "EnumVariant",
            RuntimeValue::Channel(_)      => "Channel",
            RuntimeValue::ChannelSender(_) => "ChannelSender",
        }
    }
}

// We need PartialEq for tests; use a simple structural implementation.
impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeValue::Integer(a),  RuntimeValue::Integer(b))  => a == b,
            (RuntimeValue::Float(a),    RuntimeValue::Float(b))    => a == b,
            (RuntimeValue::String(a),   RuntimeValue::String(b))   => a == b,
            (RuntimeValue::Boolean(a),  RuntimeValue::Boolean(b))  => a == b,
            (RuntimeValue::Null,        RuntimeValue::Null)        => true,
            (RuntimeValue::Moved,       RuntimeValue::Moved)       => true,
            (RuntimeValue::Error(a),    RuntimeValue::Error(b))    => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Integer(val)  => write!(f, "{val}"),
            RuntimeValue::Float(val)    => write!(f, "{val}"),
            RuntimeValue::String(val)   => write!(f, "{val}"),
            RuntimeValue::Boolean(val)  => write!(f, "{val}"),
            RuntimeValue::Null          => write!(f, "null"),
            RuntimeValue::Moved         => write!(f, "<moved>"),
            RuntimeValue::Error(msg)    => write!(f, "Error({msg})"),
            RuntimeValue::Function(func)        => write!(f, "<function {}>", func.name),
            RuntimeValue::Class(c)              => write!(f, "<class {}>", c.name),
            RuntimeValue::BoundMethod { method, .. } => write!(f, "<method {}>", method.name),
            RuntimeValue::Builtin { name, .. }  => write!(f, "<builtin {name}>"),
            RuntimeValue::Channel(_)            => write!(f, "<channel>"),
            RuntimeValue::ChannelSender(_)      => write!(f, "<channel_sender>"),
            RuntimeValue::EnumVariant { enum_name, variant, payload } => {
                if let Some(p) = payload {
                    write!(f, "{enum_name}::{variant}({p})")
                } else {
                    write!(f, "{enum_name}::{variant}")
                }
            }
            RuntimeValue::Array(arr) => {
                let b = arr.borrow();
                write!(f, "[")?;
                for (i, v) in b.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            RuntimeValue::Map(map) => {
                let b = map.borrow();
                write!(f, "{{")?;
                let mut keys: Vec<&String> = b.keys().collect();
                keys.sort();
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{k}: {}", b.get(*k).unwrap())?;
                }
                write!(f, "}}")
            }
            RuntimeValue::Object { class_name, fields, .. } => {
                let fb = fields.borrow();
                if fb.is_empty() {
                    write!(f, "{class_name} {{}}")
                } else {
                    write!(f, "{class_name} {{ ")?;
                    let mut sorted_keys: Vec<&String> = fb.keys().collect();
                    sorted_keys.sort();
                    for (i, key) in sorted_keys.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}: {}", key, fb.get(*key).unwrap())?;
                    }
                    write!(f, " }}")
                }
            }
            RuntimeValue::Struct { name, fields } => {
                if fields.is_empty() {
                    write!(f, "{name} {{}}")
                } else {
                    write!(f, "{name} {{ ")?;
                    let mut sorted_keys: Vec<&String> = fields.keys().collect();
                    sorted_keys.sort();
                    for (i, key) in sorted_keys.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}: {}", key, fields.get(*key).unwrap())?;
                    }
                    write!(f, " }}")
                }
            }
        }
    }
}
