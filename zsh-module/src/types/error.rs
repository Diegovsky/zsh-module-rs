use crate::variable;
use std::{env, fmt, io, path::*};

/// A zsh error meant for use in this library internally
///
/// Comes with several useful error types.
#[derive(Debug)]
pub enum Zerror {
    /// A std::io::Error wrapper
    Io(io::Error),
    /// A std::env::VarError wrapper
    Env(std::env::VarError),

    /// An error occurring when evaluating a string
    EvalError(String),
    /// An error occurring when sourcing a file
    SourceError(PathBuf),
    /// The specified file could not be found.
    FileNotFound(PathBuf),

    /// Error interacting with variables
    Var(variable::VarError),

    /// A generic conversion error. The internal String is the error message.
    Conversion(String),
}
impl std::error::Error for Zerror {}
impl fmt::Display for Zerror {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(i) => i.fmt(f),
            Self::Env(i) => i.fmt(f),

            Self::EvalError(cmd) => write!(
                f,
                "Something went wrong while evaluating the command: {}",
                cmd
            ),
            Self::SourceError(path) => write!(
                f,
                "Something went wrong while sourcing the file: {}",
                path.display()
            ),
            Self::Var(e) => write!(f, "Variable error: {}", e),
            Self::FileNotFound(path) => write!(f, "File not found: {}", path.display()),

            Self::Conversion(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}
impl From<io::Error> for Zerror {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<env::VarError> for Zerror {
    fn from(e: env::VarError) -> Self {
        Self::Env(e)
    }
}
impl From<variable::VarError> for Zerror {
    fn from(e: variable::VarError) -> Self {
        Self::Var(e)
    }
}

/// The requirements for an error to be a zsh module error
pub trait ZshModuleError: Into<Zerror> + fmt::Debug + fmt::Display {}
impl<E> ZshModuleError for E where E: Into<Zerror> + fmt::Debug + fmt::Display {}

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

/// Represents the possibility of an error `E`.
/// It is basically a [`Result`] that only cares for its [`Err`] variant.
///
/// # Generics
/// You can (and should) replace the default error type `E` with your own [`Zerror`].
pub type MaybeZerror<E = Zerror> = Result<(), E>;
