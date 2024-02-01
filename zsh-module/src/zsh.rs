//! A collection of functions used to interact directly with Zsh
use std::{
    io::Read,
    path::{Path, PathBuf},
};

use crate::{to_cstr, ErrorCode, MaybeZError, ToCString, ZError, ZErrorExt, ZResultExt};

use zsh_sys as zsys;

// #[derive(Debug)]
// pub struct InternalError;

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
pub fn eval_simple<S>(cmd: S) -> MaybeZError
where
    S: AsRef<str>,
{
    static ZSH_CONTEXT_STRING: &[u8] = b"zsh-module-rs-eval\0";
    let og_cmd = cmd.as_ref();
    unsafe {
        let cmd = to_cstr(cmd.as_ref());
        zsys::execstring(
            cmd.as_ptr() as *mut _,
            1,
            0,
            ZSH_CONTEXT_STRING.as_ptr() as *mut _,
        );
        let errflag = zsys::errflag;
        if errflag != 0 {
            Err(ZError::EvalError((
                String::from(og_cmd),
                errflag as ErrorCode,
            )))
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

pub fn source_file<P>(path: P) -> MaybeZError
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if !path.is_file() {
        // Don't source it if we know it can't be sourced
        return Err(ZError::FileNotFound(path.to_path_buf()));
    }

    let path_str = path.into_cstr();
    let result = unsafe { zsys::source(path_str.as_ptr() as *mut _) };
    if result == zsys::source_return_SOURCE_OK {
        Ok(())
    } else {
        Err(ZError::SourceError((
            path.to_path_buf(),
            result as ErrorCode,
        )))
    }
}
