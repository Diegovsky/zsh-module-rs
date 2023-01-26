use std::ptr::slice_from_raw_parts_mut;

use zsh_sys as zsys;

pub(crate) struct Features {
    pub raw: zsys::features,
}

impl std::ops::Deref for Features {
    type Target = zsys::features;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl std::ops::DerefMut for Features {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

macro_rules! feature_list_method {
    ($method:ident, $get:ident, $ty:ty, $list:ident, $size:ident) => {
        pub fn $method(mut self, slice: Box<[$ty]>) -> Self {
            let mem = Box::leak(slice);
            self.raw.$list = mem.as_mut_ptr();
            self.raw.$size = mem.len() as i32;
            self
        }
        pub fn $get<'a>(&'a mut self) -> &'a mut [$ty] {
            unsafe { std::slice::from_raw_parts_mut(self.$list, self.$size as usize) }
        }
    };
}

impl Features {
    pub fn empty() -> Self {
        unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
    }
    feature_list_method!(binaries, get_binaries, zsys::builtin, bn_list, bn_size);
    /* feature_list_method!(conddef, zsys::conddef, cd_list, cd_size);
    feature_list_method!(mathfuncs, zsys::mathfunc, mf_list, mf_size);
    feature_list_method!(paramdefs, zsys::paramdef, pd_list, pd_size); */
}

unsafe fn free_list<T: std::fmt::Debug>(data: *mut T, len: i32) {
    if data.is_null() {
        // Not initialized, so no need to be freed
        return;
    }
    let _ = Box::from_raw(slice_from_raw_parts_mut(data, len as usize));
}

impl std::ops::Drop for Features {
    fn drop(&mut self) {
        // Drop stuff that was moved
        unsafe {
            free_list(self.bn_list, self.bn_size);
            free_list(self.mf_list, self.mf_size);
            free_list(self.cd_list, self.cd_size);
            free_list(self.pd_list, self.pd_size);
        }
    }
}
