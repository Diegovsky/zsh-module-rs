//! Zsh native log functions. This module contains high level interfaces to the zsh log functions.
use std::ffi::CString;

use zsh_sys as zsys;

/// Prints out a warning message from the command `cmd`
pub fn warn_named(cmd: &str, msg: &str) {
    let cmd_c = CString::new(cmd).expect("Failed to make CStr, remove null byte");
    let msg_c = CString::new(msg).expect("Failed to make CStr, remove null byte");
    unsafe { zsys::zwarnnam(cmd_c.as_ptr(), msg_c.as_ptr()) }
}

/// Prints out a warning message
pub fn warn(msg: &str) {
    let msg_c = CString::new(msg).expect("Failed to make CStr, remove null byte");
    unsafe { zsys::zwarn(msg_c.as_ptr()) }
}

/// Prints out an error message
pub fn error(msg: &str) {
    let msg_c = CString::new(msg).expect("Failed to make CStr, remove null byte");
    unsafe { zsys::zerr(msg_c.as_ptr()) }
}

#[macro_export]
/// Prints out a warning message from a command, like [`println!`]
macro_rules! warn_named {
    ($cmd:expr, $msg:expr $(,$val:expr)*) => {
       $crate::log::warn_named($cmd, &format!($msg, $($val),*))
    };
}

#[macro_export]
/// Prints out a warning message, like [`println!`]
macro_rules! warn {
    ($msg:expr $(,$val:expr)*) => {
       $crate::log::warn(&format!($msg, $($val),*))
    };
}

/// Prints out an error message, like [`println!`]
#[macro_export]
macro_rules! error {
    ($msg:expr $(,$val:expr)*) => {
       $crate::log::error(&format!($msg, $($val),*))
    };
}
