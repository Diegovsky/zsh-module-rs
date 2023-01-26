//! Zsh native log functions. This module contains high level interfaces to the zsh log functions.

use std::ffi::CStr;

use zsh_sys as zsys;

use crate::cstr;

/// Prints out a warning message from the command `cmd`. See [`crate::warn_named!`]
pub fn warn_named(cmd: &str, msg: &str) {
    let cmd_c = cstr(cmd);
    let msg_c = cstr(msg);
    unsafe { zsys::zwarnnam(cmd_c.as_ptr(), msg_c.as_ptr()) }
}

/// Prints out a warning message. See [`crate::warn!`]
pub fn warn(msg: &str) {
    let msg_c = cstr(msg);
    unsafe { zsys::zwarn(msg_c.as_ptr()) }
}

/// Prints out an error message. See [`crate::error!`]
pub fn error(msg: &str) {
    let msg_c = cstr(msg);
    unsafe { zsys::zerr(msg_c.as_ptr()) }
}

/// Prints out an error message from the command `cmd`. See [`crate::error_named!`]
pub fn error_named(cmd: &str, msg: &str) {
    let cmd = cstr(cmd);
    let msg = cstr(msg);
    error_named_raw(&cmd, &msg)
}

pub(crate) fn error_named_raw(cmd: &CStr, msg: &CStr) {
    unsafe { zsys::zerrnam(cmd.as_ptr(), msg.as_ptr()) }
}

#[macro_export]
/// Prints out a warning message with a command name, like [`println!`]
/// ## Example
/// ```rust
/// fn my_cd(action: &mut Action, name: &str, args: &[&str]) -> Result<()> {
///     if args.len() > 1 {
///         zsh_module::warn_named!(name, "too much arguments!");
///     }
///    // code
///    ...
/// }
///
/// ```
macro_rules! warn_named {
    ($cmd:expr, $msg:expr $(,$val:expr)*) => {
       $crate::log::warn_named($cmd, &format!($msg, $($val),*))
    };
}

#[macro_export]
/// Prints out an error message with a command name, like [`println!`]
/// ## Example
/// ```rust
/// fn my_cd(action: &mut Action, name: &str, args: &[&str]) -> Result<()> {
///     if args.len() > 1 {
///         zsh_module::error_named!(name, "too much arguments!");
///         return Err(/* error */)
///     }
///    // code
///    ...
/// }
///
/// ```
macro_rules! error_named {
    ($cmd:expr, $msg:expr $(,$val:expr)*) => {
       $crate::log::error_named($cmd, &format!($msg, $($val),*))
    };
}

/// Prints out a warning message, like [`println!`]
/// ## Example
/// ```rust
/// let number = 10;
/// if number != 42 {
///     zsh_module::warn!("Wrong number, expected 42, got {}", number);
/// }
///
/// ```
#[macro_export]
macro_rules! warn {
    ($msg:expr $(,$val:expr)*) => {
       $crate::log::warn(&format!($msg, $($val),*))
    };
}

/// Prints out an error message, like [`println!`]
/// ## Example
/// ```rust
/// let number = 10;
/// if number != 42 {
///     zsh_module::error!("Wrong number, expected 42, got {}", number);
///     return Err(Not42Error)
/// }
///
/// ```
#[macro_export]
macro_rules! error {
    ($msg:expr $(,$val:expr)*) => {
       $crate::log::error(&format!($msg, $($val),*))
    };
}
