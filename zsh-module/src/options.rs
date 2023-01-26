use std::{ffi::CStr, os::raw::c_char};

use zsh_sys as zsys;

/// Represents all the options passed to a command.
pub struct Opts {
    raw: zsys::Options,
}

impl Opts {
    pub(crate) unsafe fn from_raw(raw: zsys::Options) -> Self {
        Self { raw }
    }
    // Taken from `zsh.h`
    // Let's hope Zsh does not change the implementation of these:

    /// Whether the option was set using a minus.
    /// E.g:
    /// ```zsh
    /// command +o # Returns false
    /// command -o # Returns true.
    /// ```
    pub fn is_minus(&self, c: c_char) -> bool {
        let expr = unsafe { (*self.raw).ind[c as usize] & 1 };
        expr != 0
    }
    /// Whether the option was set using a plus.
    /// E.g:
    /// ```zsh
    /// command +o # Returns true
    /// command -o # Returns false
    /// ```
    pub fn is_plus(&self, c: c_char) -> bool {
        let expr = unsafe { (*self.raw).ind[c as usize] & 2 };
        expr != 0
    }
    /// Whether the option was set.
    /// E.g:
    /// ```zsh
    /// command +o # Returns true
    /// command -o # Returns true
    /// command # Returns false
    /// ```
    pub fn is_set(&self, c: c_char) -> bool {
        unsafe { (*self.raw).ind[c as usize] != 0 }
    }
    /// Returns the argument passed with the option, if any.
    /// E.g:
    /// ```zsh
    /// command +o example # Returns Some("example")
    /// command -o example2 # Returns Some("example2")
    /// command -o # Returns None
    /// command # Returns None
    /// ```
    pub fn get_arg(&self, c: c_char) -> Option<&str> {
        unsafe {
            let args =
                std::ptr::slice_from_raw_parts((*self.raw).args, (*self.raw).argscount as usize);
            let opt = (*self.raw).ind[c as usize];
            if opt > 3 {
                CStr::from_ptr((*args)[(opt >> 2) as usize - 1])
                    .to_str()
                    .ok()
            } else {
                None
            }
        }
    }
}
