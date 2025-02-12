#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
pub use alloc::string::String;
#[cfg(feature = "alloc")]
pub use alloc::vec::Vec;

#[cfg(any(feature = "serde_std", feature = "serde_alloc"))]
use serde::{Deserialize, Serialize};

use sbor::describe::*;
use sbor::{Decode, Encode, TypeId};

/// Represents a blueprint.
#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize)
)]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Blueprint {
    pub package: String,
    pub name: String,
    pub functions: Vec<Function>,
    pub methods: Vec<Method>,
}

/// Represents a function.
#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize)
)]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<Type>,
    pub output: Type,
}

/// Represents a method.
#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize)
)]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Method {
    pub name: String,
    pub mutability: Mutability,
    pub inputs: Vec<Type>,
    pub output: Type,
}

/// Whether a method is going to change the component state.
#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize)
)]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Mutability {
    /// An immutable method requires an immutable reference to component state.
    Immutable,

    /// A mutable method requires a mutable reference to component state.
    Mutable,
}
