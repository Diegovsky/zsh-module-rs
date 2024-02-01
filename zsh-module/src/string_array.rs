use std::ffi::{c_char, CStr};

use crate::from_cstr;

/// A thin wrapper around a Zsh string array. A.K.A null terminated pointer of strings
/// ```no_run
/// fn cmd(data: &(), name: &str, args: zsh_module::StringArray, _opts: zsh_module::Opts) {
///     let arg_cstr: Option<&std::ffi::CStr> = args.get_cstr(0);
///     let arg: Option<Result<&str, std::str::Utf8Error>>  = args.get(0);
///     let arg: &str = &args[0];
/// }
/// ```
pub struct StringArray {
    raw: *const *const c_char,
    len: usize,
}

impl StringArray {
    pub(crate) unsafe fn from_raw(raw: *const *const c_char) -> Self {
        let mut len = 0;
        while unsafe { !raw.add(len).read().is_null() } {
            len += 1
        }
        Self { raw, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn get_cstr(&self, index: usize) -> Option<&CStr> {
        if index < self.len() {
            unsafe { from_cstr(*self.raw.add(index)) }
        } else {
            None
        }
    }
    pub fn get(&self, index: usize) -> Option<Result<&str, std::str::Utf8Error>> {
        self.get_cstr(index).map(CStr::to_str)
    }
    pub fn iter(&self) -> StrIter<'_> {
        StrIter {
            array: self,
            index: 0,
        }
    }
    pub fn iter_cstr(&self) -> CStrIter<'_> {
        CStrIter {
            array: self,
            index: 0,
        }
    }
}

impl std::ops::Index<usize> for StringArray {
    type Output = str;
    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Some(Ok(out)) => out,
            Some(Err(e)) => panic!("failed to convert c string: {}", e),
            None => panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.len(),
                index
            ),
        }
    }
}

impl std::fmt::Debug for StringArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter_cstr()).finish()
    }
}
/// An iterator that goes through a [`StringArray`] and yields [`&str`].
pub struct CStrIter<'a> {
    array: &'a StringArray,
    index: usize,
}

impl<'a> std::iter::Iterator for CStrIter<'a> {
    type Item = &'a CStr;
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.array.get_cstr(self.index);
        self.index += 1;
        val
    }
}

/// An iterator that goes through a [`StringArray`] and yields [`&str`].
pub struct StrIter<'a> {
    array: &'a StringArray,
    index: usize,
}

impl<'a> std::iter::Iterator for StrIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.array.get(self.index).and_then(Result::ok);
        self.index += 1;
        val
    }
}
