use zsh_sys as zsys;

pub(crate) struct Features {
    raw: zsys::features,
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
    ($method:ident, $ty:ty, $list:ident, $size:ident) => {
        pub fn $method(mut self, slice: Box<[$ty]>) -> Self {
            let mem = Box::leak(slice);
            self.raw.$list = mem.as_mut_ptr();
            self.raw.$size = mem.len() as i32;
            self
        }
    };
}

impl Features {
    pub fn empty() -> Self {
        unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
    }
    feature_list_method!(binaries, zsys::builtin, bn_list, bn_size);
    /* feature_list_method!(conddef, zsys::conddef, cd_list, cd_size);
    feature_list_method!(mathfuncs, zsys::mathfunc, mf_list, mf_size);
    feature_list_method!(paramdefs, zsys::paramdef, pd_list, pd_size); */
}

impl std::ops::Drop for Features {
    fn drop(&mut self) {
        unsafe fn free_list<T>(data: *mut T, len: i32) {
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(data, len as usize));
        }
        // Drop stuff that was moved
        unsafe {
            free_list(self.bn_list, self.bn_size);
            free_list(self.mf_list, self.mf_size);
            free_list(self.cd_list, self.cd_size);
            free_list(self.pd_list, self.pd_size);
        }
    }
}
