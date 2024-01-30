use std::{
    borrow::Cow,
    ffi::{c_char, CStr, CString, OsStr},
    path::*,
};

/// An internal helper function used to convert stringlikes into CStrings.
/// You will likely never need to use this directly.
pub fn to_cstr(string: impl Into<Vec<u8>>) -> CString {
    CString::new(string).expect("Strings should not contain a null byte!")
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
