//! A collection of functions used to interact directly with Zsh
use std::{
    io::Read,
    path::{Path, PathBuf},
};

use crate::{to_cstr, ErrorCode, MaybeZError, ToCString, ZError};

use parking_lot::Mutex;
use zsh_sys as zsys;

mod param;

pub use param::{Param, ParamValue};

#[derive(Debug)]
/// The error type for Zsh operations that interpret code.
pub struct InternalError;

// impl std::fmt::Display for InternalError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Something went wrong while sourcing the file")
//     }
// }
// impl std::error::Error for InternalError {}

/// Evaluates a zsh script string
/// # Examples
/// ```no_run
/// zsh_module::zsh::eval_simple("set -x").unwrap();
/// zsh_module::zsh::eval_simple("function func() { echo 'Hello from func' }").unwrap();
/// ```
///
pub fn eval_simple(cmd: impl ToCString) -> MaybeZError<InternalError> {
    static ZSH_CONTEXT_STRING: &[u8] = b"zsh-module-rs-eval\0";
    let og_cmd = cmd.as_ref();
    unsafe {
        let cmd = cmd.into_cstr();
        zsys::execstring(
            cmd.as_ptr() as *mut _,
            1,
            0,
            ZSH_CONTEXT_STRING.as_ptr() as *mut _,
        );
        let errflag = zsys::errflag;
        if errflag != 0 {
            Err(ZError::EvalError(errflag as ErrorCode))
        } else {
            Ok(())
        }
    }
}

/// Zsh's current parameter table. Allows you to inspect the values of shell variables, including
/// locals. In the future, will allow you to set them too.
/// ```no_run
/// use zsh_module::zsh::{ Param, paramtab };
/// let mut tbl = zsh_module::zsh::paramtab();
/// let zdotdir = tbl.get("ZDOTDIR").and_then(Param::as_str).unwrap();
/// println!("$ZDOTDIR: {}", zdotdir);
/// ```
pub fn paramtab() -> HashTable<Param> {
    // Prevent multiple threads from accessing this at the same time
    static PARAMS_BORROWED: Mutex<()> = parking_lot::const_mutex(());
    std::mem::forget(PARAMS_BORROWED.lock());
    unsafe { HashTable::new(zsys::paramtab, || PARAMS_BORROWED.force_unlock()) }
}

// for some shell globals, take a look at Src/init.c:source

// !TODO: implement zsh's stdin
/* pub fn stdin() -> impl Read {
    std::os::unix::io::FromRawFd::from_raw_fd(zsys::SHIN)
} */

pub fn source_file<P>(path: P) -> MaybeZError
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if !path.is_file() {
        // Don't source it if we know it can't be sourced
        // Prefer to use the internal rust ZError type for this before finding out the hard way.
        return Err(ZError::FileNotFound(path.into()));
    }

    let path_str = path.into_cstr();
    let result = unsafe { zsys::source(path_str.as_ptr() as *mut _) };
    if result == zsys::source_return_SOURCE_OK {
        Ok(())
    } else {
        Err(ZError::SourceError(result as ErrorCode))
    }
}
