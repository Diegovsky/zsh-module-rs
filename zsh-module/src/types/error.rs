use crate::variable;
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
    Return(ErrorCode),

    /// A std::io::Error wrapper. Try to avoid using this directly
    /// as a return value from your function if possible.
    Io(io::Error),

    /// A std::env::VarError wrapper. Try to avoid using this directly
    /// as a return value from your function if possible.
    Env(env::VarError),

    /// An error occurring when evaluating a string
    EvalError(ErrorCode),
    /// An error occurring when sourcing a file
    SourceError(ErrorCode),
    /// The specified file could not be found.
    FileNotFound,

    /// Error interacting with variables
    Var(variable::VarError),

    /// A generic conversion error. The internal String is the error message.
    Conversion(String),
}
impl std::error::Error for ZError {}
impl fmt::Display for ZError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(i) => write!(f, "Received return code: {i}"),
            Self::Io(e) => e.fmt(f),
            Self::Env(e) => e.fmt(f),

            Self::EvalError(e) => write!(f, "eval error: {e}"),
            Self::SourceError(e) => write!(f, "source error: {e}"),
            Self::Var(v) => v.fmt(f),
            Self::FileNotFound => "File not found".fmt(f),

            Self::Conversion(msg) => write!(f, "Conversion error: {msg}"),
        }
    }
}
impl From<env::VarError> for ZError {
    fn from(e: env::VarError) -> Self {
        Self::Env(e)
    }
}
impl From<io::Error> for ZError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<ErrorCode> for ZError {
    fn from(e: ErrorCode) -> Self {
        Self::Return(e)
    }
}
impl From<variable::VarError> for ZError {
    fn from(e: variable::VarError) -> Self {
        Self::Var(e)
    }
}

/// Represents the possibility of a zerror.
/// Only use this for functions that aren't expected to return anything.
pub type MaybeZError = Result<(), ZError>;

/// A [`Result`] wrapper around [`ZError`].
pub type ZResult<T> = Result<T, ZError>;
