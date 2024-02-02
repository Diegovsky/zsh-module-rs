use std::{mem, ops::{Deref, DerefMut}, ptr::NonNull};

///! This module implements a bridge to Zsh's memory allocation facilities.

use zsh_sys as zsys;

/// A value allocated using Zsh's internal allocator API. This is useful when you want to store a
/// value as a param, for example.
#[repr(transparent)]
pub struct ZBox<T>(std::ptr::NonNull<T>);

impl<T> ZBox<T> {
    /// Allocates a value using Zsh's internal allocator API.
    pub fn new(val: T) -> Self {
        let ptr = unsafe { zsys::zalloc(mem::size_of::<T>()) };
        let ptr = NonNull::new(ptr.cast::<T>()).unwrap();
        unsafe { ptr.as_ptr().write(val) };
        Self(ptr)
    }
}

impl<T> Drop for ZBox<T> {
    fn drop(&mut self) {
        unsafe {
            zsys::zfree(self.0.as_ptr().cast(), mem::size_of::<T>() as i32)
        }
    }
}

impl<T> std::fmt::Debug for ZBox<T> where T: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ZBox").field(&*self).finish()
    }
}

impl<T> Deref for ZBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
       unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for ZBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
       unsafe { self.0.as_mut() }
    }
}
