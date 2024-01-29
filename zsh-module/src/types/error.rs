// use crate::variable;
use std::{env, ffi, fmt, io, path::*};

/// The internal error code type.
pub type ErrorCode = isize;

// TODO: Rewrite all doc comments to use new API stuff
/// A zsh error meant for use in this library internally
///
/// Comes with several useful error types.
#[derive(Debug)]
pub enum ZError {
    /// A low-level generic return type for zsh internal functions that return integer return types
    ///
    /// TODO: Rewrite zsh-sys stuff to use this (if a better solution cannot be implemented)
    Other(ErrorCode),

    /// An error occurring when evaluating a string
    EvalError(ErrorCode),
    /// An error occurring when sourcing a file
    SourceError(ErrorCode),
    /// The specified file could not be found.
    FileNotFound(PathBuf),

    // /// Error interacting with variables
    // Var(variable::VarError),
    /// A generic conversion error. The internal String is the error message.
    Conversion(String),
}
impl std::error::Error for ZError {}
impl fmt::Display for ZError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other(i) => write!(f, "Received return code: {i}"),

            Self::EvalError(e) => write!(f, "eval error: {e}"),
            Self::SourceError(e) => write!(f, "source error: {e}"),
            // Self::Var(v) => v.fmt(f),
            Self::FileNotFound(p) => write!(f, "File not found: {}", p.display()),

            Self::Conversion(msg) => write!(f, "Conversion error: {msg}"),
        }
    }
}
impl From<ErrorCode> for ZError {
    fn from(e: ErrorCode) -> Self {
        Self::Other(e)
    }
}
// impl From<variable::VarError> for ZError {
//     fn from(e: variable::VarError) -> Self {
//         Self::Var(e)
//     }
// }

/// Represents the possibility of a zerror.
/// Only use this for functions that aren't expected to return anything.
pub type MaybeZError = Result<(), ZError>;

/// A [`Result`] wrapper around [`ZError`].
pub type ZResult<T> = Result<T, ZError>;
