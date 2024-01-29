use std::os::raw::c_char;

use zsh_sys as zsys;

use crate::to_cstr;

/// A wrapper around Zsh's hashtable implementation
///
/// TODO: Finish this
#[derive(Debug)]
pub struct HashTable {
    raw: zsys::HashTable,
}

impl HashTable {
    pub(crate) unsafe fn from_raw(raw: zsys::HashTable) -> Self {
        Self { raw }
    }
    pub(crate) fn get(&self, name: &str) -> zsys::HashNode {
        let name = to_cstr(name);
        unsafe { self.raw_get(name.as_ptr()) }
    }
    pub(crate) fn remove(&self, name: &str) -> zsys::HashNode {
        let name = to_cstr(name);
        unsafe { self.raw_remove(name.as_ptr()) }
    }
    pub(crate) unsafe fn raw_get(&self, name: *const c_char) -> zsys::HashNode {
        zsys::gethashnode(self.raw, name)
    }
    pub(crate) unsafe fn raw_remove(&self, name: *const c_char) -> zsys::HashNode {
        zsys::removehashnode(self.raw, name)
    }
}
