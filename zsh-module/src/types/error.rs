use crate::variable;
use std::{env, fmt, io, path::*};

/// The internal error code type.
pub type ErrorCode = isize;

// TODO: Rewrite all doc comments to use new API stuff
/// A zsh error meant for use in this library internally
///
/// Comes with several useful error types.
#[derive(Debug)]
pub enum ZError {
    /// A low-level return type for zsh internal functions that return integer return types
    ///
    /// TODO: Rewrite zsh-sys stuff to use this (if a better solution cannot be implemented)
    Return(ErrorCode),

    /// A std::io::Error wrapper, including the filepath that caused the error
    Io((PathBuf, io::Error)),

    // /// A std::io::Error wrapper designed for use by library users. Please do not use this.
    // RawIo(io::Error),
    /// A std::env::VarError wrapper
    Env((String, env::VarError)),

    /// An error occurring when evaluating a string
    EvalError((String, ErrorCode)),
    /// An error occurring when sourcing a file
    SourceError((PathBuf, ErrorCode)),
    /// The specified file could not be found.
    FileNotFound(PathBuf),

    /// Error interacting with variables
    Var(variable::VarError),

    /// A generic conversion error. The internal String is the error message.
    Conversion(String),
}
impl std::error::Error for ZError {}
impl fmt::Display for ZError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(i) => write!(f, "Received return value: {i}"),
            Self::Io((p, i)) => write!(f, "Io error from filepath {}: {i}", p.display()),
            Self::Env((v, e)) => write!(f, "Var error from variable {v}: {e}"),

            Self::EvalError((c, e)) => write!(f, "Exit code {e} while evaluating the command: {c}"),
            Self::SourceError((p, e)) => {
                write!(f, "Exit code {e} while sourcing the file: {}", p.display())
            }
            Self::Var(v) => v.fmt(f),
            Self::FileNotFound(path) => write!(f, "File not found: {}", path.display()),

            Self::Conversion(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}
// impl From<io::Error> for ZError {
//     fn from(e: io::Error) -> Self {
//         Self::Io(e)
//     }
// }
// impl From<env::VarError> for ZError {
//     fn from(e: env::VarError) -> Self {
//         Self::Env(e)
//     }
// }
impl From<ErrorCode> for ZError {
    fn from(value: ErrorCode) -> Self {
        Self::Return(value)
    }
}
impl From<variable::VarError> for ZError {
    fn from(e: variable::VarError) -> Self {
        Self::Var(e)
    }
}

/// The requirements for an error to be a zsh module error
pub trait ZshModuleError: Into<ZError> + fmt::Debug + fmt::Display {}
impl<E> ZshModuleError for E where E: Into<ZError> + fmt::Debug + fmt::Display {}

/// A zsh module error meant for use in modules.
///
/// Contains convenience functions and custom error types.
#[derive(Debug, Default)]
pub enum ZshModError<E>
where
    E: ZshModuleError,
{
    /// A custom error. Use this for descriptive errors that happen upon module initialization.
    Init(E),
    /// A custom error type. Use this for errors that happen during runtime.
    Runtime(E),
    /// A custom error. Use this when you want to provide your own error message,
    /// and the other custom types don't really fit.
    Custom(String),

    /// An unknown error. Contains no helpful information. Please try not to use this if possible.
    #[default]
    Unknown,
}
impl<E> std::error::Error for ZshModError<E> where E: ZshModuleError {}
impl<E> fmt::Display for ZshModError<E>
where
    E: ZshModuleError,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init(e) => write!(f, "Initialization error: {}", e),
            Self::Runtime(e) => write!(f, "Runtime error: {}", e),
            Self::Custom(e) => e.fmt(f),
            Self::Unknown => "Unknown error".fmt(f),
        }
    }
}

/// Represents the possibility of a zerror.
/// Only use this for functions that aren't expected to return anything.
pub type MaybeZerror = Result<(), ZError>;

/// A [`Result`] wrapper around [`ZError`].
pub type ZResult<T> = Result<T, ZError>;

/// A trait to facilitate converting lossy error types like [`io::Error`] and [`env::VarError`] into [`ZError`].
pub trait ZErrorExt {
    /// The type of input that may have caused this error. For example, a [`PathBuf`] for an [`io::Error`], or a [`String`] for an [`env::VarError`].
    type OffendingInput;
    /// Takes this error and the thing that caused it, and turns it into a valid [`ZError`].
    fn into_zerror(self, input: Self::OffendingInput) -> ZError;
}

impl ZErrorExt for io::Error {
    type OffendingInput = PathBuf;
    fn into_zerror(self, input: Self::OffendingInput) -> ZError {
        ZError::Io((input.into(), self))
    }
}
impl ZErrorExt for env::VarError {
    type OffendingInput = String;
    fn into_zerror(self, input: Self::OffendingInput) -> ZError {
        ZError::Env((input, self))
    }
}
