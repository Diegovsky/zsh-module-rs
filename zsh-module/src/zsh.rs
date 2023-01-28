//! A collection of functions used to interact directly with Zsh
use std::{io::Read, path::Path};

use crate::{to_cstr, MaybeError, ToCString};

use zsh_sys as zsys;

#[derive(Debug)]
pub struct InternalError;

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Something went wrong while sourcing the file")
    }
}
impl std::error::Error for InternalError {}

/// Evaluates a zsh script string
/// # Examples
/// ```no_run
/// zsh_module::zsh::eval_simple("set -x").unwrap();
/// zsh_module::zsh::eval_simple("function func() { echo 'Hello from func' }").unwrap();
/// ```
///
pub fn eval_simple(cmd: &str) -> MaybeError<InternalError> {
    static ZSH_CONTEXT_STRING: &[u8] = b"zsh-module-rs-eval\0";
    unsafe {
        let cmd = to_cstr(cmd);
        zsys::execstring(
            cmd.as_ptr() as *mut _,
            1,
            0,
            ZSH_CONTEXT_STRING.as_ptr() as *mut _,
        );
        if zsys::errflag != 0 {
            Err(InternalError)
        } else {
            Ok(())
        }
    }
}

// for some shell globals, take a look at Src/init.c:source

// !TODO: implement zsh's stdin
/* pub fn stdin() -> impl Read {
    std::os::unix::io::FromRawFd::from_raw_fd(zsys::SHIN)
} */

#[derive(Debug)]
#[repr(u32)]
pub enum SourceError {
    NotFound,
    InternalError(InternalError),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "File not found"),
            Self::InternalError(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for SourceError {}

pub fn source_file(path: impl ToCString) -> MaybeError<SourceError> {
    let path = path.into_cstr();
    let result = unsafe { zsys::source(path.as_ptr() as *mut _) };
    if result == zsys::source_return_SOURCE_OK {
        Ok(())
    } else {
        Err(match result {
            zsys::source_return_SOURCE_NOT_FOUND => SourceError::NotFound,
            zsys::source_return_SOURCE_ERROR => SourceError::InternalError(InternalError),
            _ => unreachable!(),
        })
    }
}
