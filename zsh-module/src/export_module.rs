use std::{
    ffi::{c_char, c_int, CStr},
    sync::atomic::AtomicBool,
};

use crate::{log, options::Opts, to_cstr, MaybeZError, Module};

use parking_lot::Mutex;
use zsh_sys as zsys;

struct ModuleHolder {
    module: Mutex<Option<Module>>,
    panicked: AtomicBool,
    name: Mutex<Option<&'static str>>,
}

impl ModuleHolder {
    const fn empty() -> Self {
        Self {
            module: parking_lot::const_mutex(None),
            panicked: AtomicBool::new(false),
            name: parking_lot::const_mutex(None),
        }
    }
}

// This struct is neither of them, but since it isn't exposed to user code
// and it isn't given to any threads, this should be safe.
unsafe impl Sync for ModuleHolder {}
unsafe impl Send for ModuleHolder {}

static MODULE: ModuleHolder = ModuleHolder::empty();

extern "C" fn builtin_callback(
    name: *mut c_char,
    args: *mut *mut c_char,
    opts: *mut zsys::options,
    _: i32,
) -> i32 {
    handle_panic(|| {
        let args = unsafe { crate::StringArray::from_raw(std::mem::transmute(args)) };
        let name = unsafe { CStr::from_ptr(name) };
        let opts = unsafe { Opts::from_raw(opts) };

        let mut module = get_mod();
        let Module {
            bintable,
            user_data,
            ..
        } = &mut *module;
        let bin = bintable.get_mut(name).expect("Failed to find binary name");
        match bin(
            &mut **user_data,
            name.to_str().expect("Failed to parse binary name"),
            args,
            opts,
        ) {
            Ok(()) => 0,
            Err(e) => {
                let msg = to_cstr(e.to_string());
                log::warn_named(name, msg);
                1
            }
        }
    })
    .unwrap_or(65)
}

pub fn set_name(name: &'static str) {
    MODULE.name.lock().insert(name);
}

pub fn set_mod(mut module: Module) {
    for x in module.features.get_binaries() {
        x.handlerfunc = Some(builtin_callback)
    }
    *MODULE.module.lock() = Some(module);
}

fn get_mod() -> parking_lot::MappedMutexGuard<'static, Module> {
    parking_lot::MutexGuard::map(MODULE.module.lock(), |opt| {
        opt.as_mut().expect("No module set")
    })
}

impl ModuleHolder {
    fn drop_mod(&self) {
        if !self.panicked() {
            self.module.lock().take();
        }
    }

    fn get_name(&self) -> Option<&str> {
        *self.name.lock()
    }

    fn panicked(&self) -> bool {
        self.panicked.load(std::sync::atomic::Ordering::Acquire)
    }
}

pub fn handle_maybe_error<E>(error: Result<(), E>) -> i32
where
    E: std::fmt::Display,
{
    match error {
        Ok(()) => 0,
        Err(e) => {
            let msg = e.to_string();
            if let Some(name) = MODULE.get_name() {
                crate::log::error_named(name, msg);
            } else {
                crate::log::error(msg);
            }
            1
        }
    }
}

pub fn handle_panic<F, R>(cb: F) -> Option<R>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    let res = std::panic::catch_unwind(|| cb());
    match res {
        Ok(ret) => Some(ret),
        Err(err) => {
            // Try to get the name but fallback to a generic name
            let name = MODULE.get_name().unwrap_or("Module");
            MODULE
                .panicked
                .store(true, std::sync::atomic::Ordering::Release);
            if let Some(msg) = err.downcast_ref::<&str>() {
                crate::error!("{:?} Panic: {}", name, msg);
            } else if let Some(msg) = err.downcast_ref::<String>() {
                crate::error!("{:?} Panic: {}", name, msg);
            } else {
                crate::error!("{:?} Panic: No additional information", name);
            }
            None
        }
    }
}

pub use paste;

pub mod ffi {
    pub use super::zsys::Module;
}

#[macro_export]
/// Exports a `setup` function to be called when the module needs to be set up.
/// You need to specify your module's loadable name
macro_rules! export_module {
    ($module_name:ident, $setupfn:ident) => {
        #[doc(hidden)]
        static MOD_NAME: &'static str = stringify!($module_name);

        #[no_mangle]
        #[doc(hidden)]
        extern "C" fn setup_(_: $crate::export_module::ffi::Module) -> i32 {
            $crate::export_module::handle_panic(|| {
                let res = $setupfn().map(|module| {
                    $crate::export_module::set_name(MOD_NAME);
                    $crate::export_module::set_mod(module)
                }
                );
                $crate::export_module::handle_maybe_error(res)
            })
            .unwrap_or(65)
        }

        mod _zsh_private_glue {
            use ::std::ffi::{ c_char, c_int };
            $crate::export_module!(@fn boot_(module: $crate::export_module::ffi::Module));
            $crate::export_module!(@fn features_(module: $crate::export_module::ffi::Module, features_ptr: *mut *mut *mut c_char));
            $crate::export_module!(@fn enables_(module: $crate::export_module::ffi::Module, enables_ptr: *mut *mut c_int));
            $crate::export_module!(@fn cleanup_(module: $crate::export_module::ffi::Module));
            $crate::export_module!(@fn finish_(module: $crate::export_module::ffi::Module) );
        }
    };
    (@fn $name:ident ($($arg:ident : $type:ty),*)) => {
        #[no_mangle]
        #[doc(hidden)]
        extern "C" fn $name($($arg: $type),*) -> i32 {
            $crate::export_module::$name($($arg),*)
        }
    }
}

macro_rules! mod_fn {
    (fn $name:ident($mod:ident $(,$arg:ident : $type:ty)*) try $block:expr) => {
        mod_fn!(
            fn $name($mod $(,$arg : $type)*) {
                handle_maybe_error($block)
            }
        );
    };
    (fn $name:ident($mod:ident $(,$arg:ident : $type:ty)*) $block:expr) => {
        pub fn $name($mod: $crate::zsys::Module $(,$arg: $type)*) -> i32 {
            handle_panic(|| {
                $block
            }).unwrap_or(65)
        }
    };
}

mod_fn!(
    fn boot_(_mod) try {
        // zsys::addwrapper()
        Ok::<_, std::convert::Infallible>(())
    }
);

mod_fn!(
    fn features_(mod_, features_ptr: *mut *mut *mut c_char) {
        let mut module = get_mod();
        unsafe { *features_ptr = zsys::featuresarray(mod_, &mut *module.features) };
        0
    }
);

mod_fn!(
    fn enables_(mod_, enables_ptr: *mut *mut c_int) {
        let mut module = get_mod();
        unsafe {
            zsys::handlefeatures(mod_, &mut *module.features, enables_ptr)
        }
    }
);

// Called when cleaning the module up.
mod_fn!(
    fn cleanup_(_mod) {
        let mut module = get_mod();
        unsafe {
            zsys::setfeatureenables(_mod, &mut *module.features, std::ptr::null_mut())
        }
    }
);

// Called after cleanup and when module fails to load.
mod_fn!(
    fn finish_(_mod) try {
        MODULE.drop_mod();
        Ok::<(), std::convert::Infallible>(())
    }
);
