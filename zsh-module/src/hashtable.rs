use std::{ffi::c_void, marker::PhantomData, os::raw::c_char};

use zsh_sys as zsys;

use crate::ToCString;

/// A wrapper around Zsh's hashtable implementation
///
/// TODO: Finish this
pub(crate) struct RawHashTable {
    raw: zsys::HashTable,
}

impl RawHashTable {
    pub(crate) unsafe fn from_raw(raw: zsys::HashTable) -> Self {
        Self { raw }
    }
    pub(crate) unsafe fn insert(&self, name: *mut c_char, node: *mut c_void) {
        let addnode = ((*self.raw).addnode).expect("Hashtable does not support operation");
        addnode(self.raw, name, node)
    }
    pub(crate) unsafe fn get_node<V>(&self, name: *const c_char) -> *mut V {
        let getnode = ((*self.raw).getnode).expect("Hashtable does not support operation");
        let node = getnode(self.raw, name);
        std::mem::transmute(node)
    }
    pub(crate) unsafe fn remove<V>(&self, name: *const c_char) -> *mut V {
        let removenode = ((*self.raw).removenode).expect("Hashtable does not support operation");
        let node = removenode(self.raw, name);
        std::mem::transmute(node)
    }
    pub(crate) unsafe fn dump(&self) {
        let printnode = ((*self.raw).printnode).expect("Hashtable does not support operation");
        zsys::scanhashtable(
            self.raw,
            1,
            0,
            0,
            Some(printnode),
            (zsys::PRINT_TYPE | zsys::PRINT_TYPESET) as i32,
        );
    }
}

/* #[repr(C)]
struct HashNode<V> {
    node: zsys::hashnode,
    raw: V
}

impl<V> HashNode<V> {
    pub(crate) unsafe fn from_raw(raw: zsys::HashNode) -> *mut Self {
        std::mem::transmute(raw)
    }
} */

pub struct HashTable<V> {
    raw: RawHashTable,
    drop: Option<Box<dyn FnOnce() + 'static>>,
    phantom: PhantomData<V>,
}

impl<V> std::ops::Drop for HashTable<V> {
    fn drop(&mut self) {
        if let Some(drop) = self.drop.take() {
            drop()
        }
    }
}

impl<V> HashTable<V> {
    pub(crate) unsafe fn new(raw: zsys::HashTable, drop: impl FnOnce() + 'static) -> Self {
        Self {
            raw: RawHashTable::from_raw(raw),
            phantom: PhantomData,
            drop: Some(Box::new(drop)),
        }
    }
    pub fn get(&mut self, name: impl ToCString) -> Option<&mut V> {
        let name = name.into_cstr();
        let ptr = unsafe { self.raw.get_node::<V>(name.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(&mut *ptr) }
        }
    }
    pub fn dump(&self) {
        unsafe { self.raw.dump() }
    }
}
