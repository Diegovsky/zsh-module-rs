//! Zsh native log functions. This module contains high level interfaces to the zsh log functions.

use zsh_sys as zsys;

use crate::ToCString;

/// Prints out a warning message from the command `cmd`. See [`crate::warn_named!`]
pub fn warn_named(cmd: impl ToCString, msg: impl ToCString) {
    let cmd_c = cmd.into_cstr();
    let msg_c = msg.into_cstr();
    unsafe { zsys::zwarnnam(cmd_c.as_ptr(), msg_c.as_ptr()) }
}

/// Prints out a warning message. See [`crate::warn!`]
pub fn warn(msg: impl ToCString) {
    let msg_c = msg.into_cstr();
    unsafe { zsys::zwarn(msg_c.as_ptr()) }
}

/// Prints out an error message. See [`crate::error!`]
pub fn error(msg: impl ToCString) {
    let msg_c = msg.into_cstr();
    unsafe { zsys::zerr(msg_c.as_ptr()) }
}

/// Prints out an error message from the command `cmd`. See [`crate::error_named!`]
/// Despite the name, this should be used to log fatal errors, not common ones.
/// Use [`warn`] for common errors instead.
pub fn error_named(cmd: impl ToCString, msg: impl ToCString) {
    let cmd = cmd.into_cstr();
    let msg = msg.into_cstr();
    unsafe { zsys::zerrnam(cmd.as_ptr(), msg.as_ptr()) }
}

#[macro_export]
/// Prints out a warning message with a command name, like [`println!`]
/// # Example
/// ```no_run
/// fn my_cd(action: &mut (), name: &str, args: &[&str]) -> zsh_module::MaybeError {
///     if args.len() > 1 {
///         zsh_module::warn_named!(name, "too much arguments!");
///     }
///     todo!()
/// }
///
/// ```
macro_rules! warn_named {
    ($cmd:expr, $msg:expr $(,$val:expr)*) => {
       $crate::log::warn_named($cmd, format!($msg, $($val),*))
    };
}

#[macro_export]
/// Prints out an error message with a command name, like [`println!`]
/// # Example
/// ```no_run
/// fn my_cd(action: &mut (), name: &str, args: &[&str]) -> zsh_module::MaybeError {
///     if args.len() > 1 {
///         zsh_module::error_named!(name, "too much arguments!");
///         return Err(todo!())
///     }
///    // code
///    todo!()
/// }
///
/// ```
macro_rules! error_named {
    ($cmd:expr, $msg:expr $(,$val:expr)*) => {
       $crate::log::error_named($cmd, format!($msg, $($val),*))
    };
}

/// Prints out a warning message, like [`println!`]
/// # Example
/// ```no_run
/// let number = 10;
/// if number != 42 {
///     zsh_module::warn!("Wrong number, expected 42, got {}", number);
/// }
///
/// ```
#[macro_export]
macro_rules! warn {
    ($msg:expr $(,$val:expr)*) => {
       $crate::log::warn(format!($msg, $($val),*))
    };
}

/// Prints out an error message, like [`println!`]
/// # Example
/// ```no_run
/// let number = 10;
/// if number != 42 {
///     zsh_module::error!("Wrong number, expected 42, got {}", number);
///     return /* error */
/// }
///
/// ```
#[macro_export]
macro_rules! error {
    ($msg:expr $(,$val:expr)*) => {
       $crate::log::error(format!($msg, $($val),*))
    };
}
