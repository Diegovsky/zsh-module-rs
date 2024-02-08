use std::{
    borrow::Cow,
    ffi::{c_char, CStr, CString, OsStr},
    path::*,
};

/// An internal helper function used to convert stringlikes into CStrings.
pub(crate) fn to_cstr(string: impl Into<Vec<u8>>) -> CString {
    CString::new(string).expect("Strings should not contain a null byte!")
}

pub(crate) unsafe fn from_cstr<'a>(ptr: *const c_char) -> Option<&'a CStr> {
    if ptr.is_null() {
        None
    } else {
        Some(std::ffi::CStr::from_ptr(ptr))
    }
}

pub(crate) unsafe fn str_from_cstr<'a>(ptr: *const c_char) -> Option<&'a str> {
    from_cstr(ptr).and_then(|cstr| cstr.to_str().ok())
}

/// Represents any type that can be represented as a C String. You shouldn't
/// need to implement this yourself as the most commonly used `string`-y types
/// already have this implemented.
///
/// # Examples
/// ```
/// use std::ffi::{CString, CStr};
/// use std::borrow::Cow;
///
/// use zsh_module::ToCString;
///
/// let cstr = CStr::from_bytes_with_nul(b"Hello, world!\0").unwrap();
/// let cstring = CString::new("Hello, world!").unwrap();
///
/// assert!(matches!(cstr.into_cstr(), Cow::Borrowed(data) if data == cstr));
///
/// let string = "Hello, world!";
/// assert!(matches!(ToCString::into_cstr(string), Cow::Owned(cstring)));
/// ```
pub trait ToCString {
    fn into_cstr<'a>(self) -> Cow<'a, CStr>
    where
        Self: 'a;
}

macro_rules! impl_tocstring {
    ($($type:ty),*) => {
        $(impl ToCString for $type {
            fn into_cstr<'a>(self) -> Cow<'a, CStr> where Self: 'a {
                Cow::Owned(to_cstr(self))
            }
        })*
    };
}

impl_tocstring!(Vec<u8>, &[u8], &str, String);

impl ToCString for &OsStr {
    fn into_cstr<'a>(self) -> Cow<'a, CStr>
    where
        Self: 'a,
    {
        Cow::Owned(to_cstr(self.to_string_lossy().as_bytes().to_vec()))
    }
}

impl ToCString for &Path {
    fn into_cstr<'a>(self) -> Cow<'a, CStr>
    where
        Self: 'a,
    {
        self.as_os_str().into_cstr()
    }
}

impl ToCString for &CStr {
    fn into_cstr<'a>(self) -> Cow<'a, CStr>
    where
        Self: 'a,
    {
        Cow::Borrowed(self)
    }
}

impl ToCString for CString {
    fn into_cstr<'a>(self) -> Cow<'a, CStr> {
        Cow::Owned(self)
    }
}

impl ToCString for *const c_char {
    fn into_cstr<'a>(self) -> Cow<'a, CStr> {
        Cow::Borrowed(unsafe { CStr::from_ptr(self) })
    }
}

impl ToCString for *mut c_char {
    fn into_cstr<'a>(self) -> Cow<'a, CStr> {
        Cow::Borrowed(unsafe { CStr::from_ptr(self) })
    }
}

/// A convenient wrapper around a Rust-allocated `CString` to allow mutation for C functions that
/// need it.
///
/// Some `zsh` functions mutate the input string.
#[repr(transparent)]
pub(crate) struct ManagedCStr(*mut c_char);

impl ManagedCStr {
    #[inline]
    pub fn new(c_str: impl ToCString) -> Self {
        Self(c_str.into_cstr().into_owned().into_raw())
    }
    #[inline]
    pub fn c_str(&self) -> &CStr {
        // SAFETY: since this originated from `CString`, it's always safe to call this
        unsafe { CStr::from_ptr(self.0) }
    }
    #[inline]
    pub fn ptr(&mut self) -> *mut c_char {
        self.0
    }
}

impl Drop for ManagedCStr {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.0) };
    }
}
