use std::ffi::{c_char, c_int, CStr};

use crate::{Module, Result};

use zsh_sys as zsys;

static mut MODULE: Option<Module> = None;

unsafe fn strings_from_ptr<'a>(mut ptr: *const *const c_char) -> Vec<&'a str> {
    let mut vec = Vec::with_capacity(2);
    loop {
        if *ptr == std::ptr::null() {
            break vec;
        }
        vec.push(CStr::from_ptr(*ptr).to_str().expect("Failed to parse arg"));
        ptr = ptr.add(1);
    }
}

#[no_mangle]
unsafe extern "C" fn zsh_private_callback(
    name: *mut c_char,
    args: *mut *mut c_char,
    _opts: *mut zsys::options,
    _: i32,
) -> i32 {
    let args = unsafe { strings_from_ptr(std::mem::transmute(args)) };
    let name = unsafe { CStr::from_ptr(name) };
    let mod_ = get_mod();
    let bin = mod_
        .bintable
        .get_mut(name)
        .expect("Failed to find binary name");
    match bin(
        &mut *mod_.actions,
        name.to_str().expect("Failed to parse binary name"),
        &args,
    ) {
        Ok(()) => 0,
        Err(e) => {
            crate::error!("Error: {}", e);
            1
        }
    }
}

fn get_optmod() -> &'static mut Option<Module> {
    unsafe { &mut MODULE }
}

fn get_mod() -> &'static mut Module {
    get_optmod().as_mut().expect("Module not set")
}

extern {
    // This is most likely fine, because it uses the Rust calling convention.
    // Nothing crashed and the world is still the same, so I'm 99% sure this is ok.
    #[allow(improper_ctypes)]
    fn __zsh_rust_setup() -> Result<Module>;
}

#[macro_export]
macro_rules! export_module {
    ($name:ident) => {
        #[no_mangle]
        fn __zsh_rust_setup() -> Result<Module> {
            $name()
        }
    };
}

macro_rules! mod_fn {
    (fn $name:ident($mod:ident $(,$arg:ident : $type:ty)*) try $block:expr) => {
        mod_fn!(
            fn $name($mod $(,$arg : $type)*) {
                match $block {
                    Ok(()) =>  0,
                    Err(e) => { $crate::error!("Error: {}", e); 1 },
                }
            }
        );
    };
    (fn $name:ident($mod:ident $(,$arg:ident : $type:ty)*) $block:expr) => {
        #[no_mangle]
        extern "C" fn $name(raw: $crate::zsys::Module $(,$arg: $type)*) -> i32 {
            let res = std::panic::catch_unwind(|| {
                let $mod = raw;
                $block
            });
            match res {
                Ok(ret) => ret,
                Err(err) => {
                    if let Some(msg) = err.downcast_ref::<&str>() {
                        $crate::error!("Rust Panic: {}", msg);
                    } else if let Some(msg) = err.downcast_ref::<String>() {
                        $crate::error!("Rust Panic: {}", msg);
                    } else {
                        $crate::error!("Rust Panic: No additional information");
                    }
                    65
                }
            }
        }
    };
}

mod_fn!(
    fn setup_(_mod) {
        let module = match unsafe { __zsh_rust_setup() } {
            Ok(module) => module,
            Err(e) => {
                crate::error!("Failed to setup module: {}", e);
                return 1
            }
        };
        for i in 0..module.features.bn_size {
            unsafe { (*module.features.bn_list.add(i as usize)).handlerfunc = Some(zsh_private_callback) }
        }
        *get_optmod() = Some(module);
        0
    }
);

mod_fn!(
    fn boot_(_mod) try {
        get_mod().actions.boot()
    }
);

mod_fn!(
    fn features_(mod_, features_ptr: *mut *mut *mut c_char) {
        let module = get_mod();
        unsafe { *features_ptr = zsys::featuresarray(mod_, &mut *module.features) };
        0
    }
);

mod_fn!(
    fn enables_(mod_, enables_ptr: *mut *mut c_int) {
        let module = get_mod();
        unsafe {
            zsys::handlefeatures(mod_, &mut *module.features, enables_ptr)
        }
    }
);

// Called when cleaning the module up.
mod_fn!(
    fn cleanup_(_mod) try {
        get_mod().actions.cleanup()
    }
);

// Called after cleanup and when module fails to load.
mod_fn!(
    fn finish_(_mod) try {
        get_optmod().take();
        Ok::<(), std::convert::Infallible>(())
    }
);
