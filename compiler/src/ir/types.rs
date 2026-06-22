//! IR type definitions for the Bunzo compiler.
//!
//! [`IrType`] represents the Bunzo-level type of every IR value.
//! Types are deliberately platform-independent — they carry no
//! information about machine word sizes, calling conventions, or
//! memory layouts. A future backend (LLVM, WASM, …) is responsible
//! for mapping these to concrete target types.

use std::fmt;

/// The Bunzo IR type of a value or instruction result.
///
/// Every virtual register in the IR has an associated `IrType`.
/// This enables a future optimiser or backend to reason about types
/// without re-visiting the original AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IrType {
    /// 64-bit signed integer (`let x = 42`).
    Int,

    /// 64-bit IEEE 754 float (`let x = 3.14`).
    Float,

    /// Heap-allocated UTF-8 string (`let x = "hello"`).
    String,

    /// Boolean (`true` / `false`).
    Bool,

    /// The `null` value.
    Null,

    /// A homogeneous array of another type (`[1, 2, 3]`).
    Array(Box<IrType>),

    /// A named struct type.
    Struct(std::string::String),

    /// A named class type.
    Class(std::string::String),

    /// A function type with parameter types and a return type.
    Function {
        params: Vec<IrType>,
        return_type: Box<IrType>,
    },

    /// No value — used as the return type of `void` functions.
    Void,

    /// Unknown or dynamically-typed value.
    ///
    /// Used when the lowering pass cannot determine a precise type
    /// (e.g. for values that the runtime resolves). A future type-
    /// propagation pass may refine these.
    Any,
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrType::Int => write!(f, "int"),
            IrType::Float => write!(f, "float"),
            IrType::String => write!(f, "string"),
            IrType::Bool => write!(f, "bool"),
            IrType::Null => write!(f, "null"),
            IrType::Array(inner) => write!(f, "Array<{inner}>"),
            IrType::Struct(name) => write!(f, "struct {name}"),
            IrType::Class(name) => write!(f, "class {name}"),
            IrType::Function {
                params,
                return_type,
            } => {
                write!(f, "func(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ") -> {return_type}")
            }
            IrType::Void => write!(f, "void"),
            IrType::Any => write!(f, "any"),
        }
    }
}
