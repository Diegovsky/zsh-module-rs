//! This module defines utilities to work with C's string arrays.
//!
//! This crate strives to implement things as lightweight and simple as possible and it reflects
//! on the API.
//!
//! ```no_run
//!    fn use_cstr_array(cstr_array: CStrArray) {
//!        let cstr_array;
//!        let arg: Option<&std::ffi::CStr> = args.get(0);
//!        let arg_str: Option<Result<&str, std::str::Utf8Error>>  = args.get_str(0);
//!        let arg: &CStr = &args[0];
//!    }
//! ```
//!
use std::{
    ffi::{c_char, CStr},
    marker::PhantomData,
    ops::Deref,
};

/// A thin wrapper around a C string array. A.K.A null terminated pointer of strings
/// ```no_run
/// fn cmd(data: &(), name: &str, args: zsh_module::CStrArray, _opts: zsh_module::Opts) {
///     let arg: Option<&std::ffi::CStr> = args.get(0);
///     let arg_str: Option<Result<&str, std::str::Utf8Error>>  = args.get_str(0);
///     let arg: &CStr = &args[0];
/// }
/// ```
///
/// This is guaranteed to be represented as a pointer of pointers. And as such, all operations are
/// linear.
#[repr(transparent)]
pub struct CStrArray(*const *const c_char);

impl CStrArray {
    #[inline]
    /// Creates a [`CStrArray`] from a raw pointer. This function assumes the pointer is valid.
    pub(crate) unsafe fn from_raw(raw: *const *const c_char) -> Self {
        Self(raw)
    }

    /// Returns the amount of elements contained within this array.
    pub fn len(&self) -> usize {
        let mut len = 0;
        while unsafe { !self.0.add(len).read().is_null() } {
            len += 1
        }
        len
    }

    /// Returns a reference to a [`CStr`] located at that position or [`None`] if it is out of
    /// bounds.
    ///
    /// This is a linear operation.
    pub fn get(&self, index: usize) -> Option<&CStr> {
        self.iter().nth(index)
    }
    /// Returns a reference to a [`str`] located at that position or [`None`] if it is out of
    /// bounds.
    ///
    /// This is a linear operation.
    pub fn get_str(&self, index: usize) -> Option<Result<&str, std::str::Utf8Error>> {
        self.iter_str().nth(index)
    }

    /// Returns an iterator over this array.
    ///
    /// The iterator yields all elements from start to end, converted to [`str`].
    #[inline]
    pub fn iter_str(&self) -> StrIter<'_> {
        StrIter {
            cstr_iter: self.iter(),
        }
    }

    /// Returns an iterator over this array.
    ///
    /// The iterator yields all elements from start to end.
    #[inline]
    pub fn iter(&self) -> CStrIter<'_> {
        // SAFETY: this is safe because we assume `self.raw` is valid
        unsafe { CStrIter::new(self.0) }
    }
}

impl std::ops::Index<usize> for CStrArray {
    type Output = CStr;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Index out of bounds")
    }
}

impl std::fmt::Debug for CStrArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// An iterator that goes through a [`CStrArray`] and yields [`CStr`]s.
pub struct CStrIter<'a> {
    items: Option<*const *const c_char>,
    _phantom: PhantomData<&'a [&'a [c_char]]>,
}

impl<'a> CStrIter<'a> {
    /// This is `unsafe` because it assumes the `arr` pointer is valid and ends with a NULL
    pub(crate) unsafe fn new(arr: *const *const c_char) -> Self {
        Self {
            items: Some(arr),
            _phantom: PhantomData,
        }
    }
}

impl<'a> std::iter::Iterator for CStrIter<'a> {
    type Item = &'a CStr;
    fn next(&mut self) -> Option<Self::Item> {
        let item_ptr = self.items?;
        let item = unsafe { item_ptr.read() };
        if item.is_null() {
            self.items = None;
            return None;
        } else {
            unsafe {
                self.items = Some(item_ptr.add(1));
                Some(CStr::from_ptr(item))
            }
        }
    }
}

/// An iterator that goes through a [`CStrArray`] and yields [`&str`].
#[repr(transparent)]
pub struct StrIter<'a> {
    cstr_iter: CStrIter<'a>,
}

impl<'a> std::iter::Iterator for StrIter<'a> {
    type Item = Result<&'a str, std::str::Utf8Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.cstr_iter.next().map(CStr::to_str)
    }
}

/// An owned version of [`CStrArray`]. See that for how to use.
///
/// Frees memory on drop.
#[repr(transparent)]
#[doc(hidden)]
pub struct CStringArray(*mut *mut c_char);

impl std::fmt::Debug for CStringArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl CStringArray {
    pub fn into_inner(self) -> *mut *mut c_char {
        let inner = self.0;
        std::mem::forget(self);
        inner
    }
}

impl std::ops::Deref for CStringArray {
    type Target = CStrArray;
    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

impl Drop for CStringArray {
    fn drop(&mut self) {
        unsafe { zsh_sys::freearray(self.0) }
    }
}
