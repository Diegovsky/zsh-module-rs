use std::{
    ffi::{c_char, c_int, CStr},
    panic::AssertUnwindSafe,
    sync::atomic::AtomicBool,
};

use crate::{log, options::Opts, to_cstr, Module};

use parking_lot::Mutex;
use zsh_sys as zsys;

#[doc(hidden)]
pub static MODULE: ModuleHolder = ModuleHolder::empty();

#[doc(hidden)]
pub struct ModuleHolder {
    module: Mutex<Option<Module>>,
    panicked: AtomicBool,
    name: Mutex<Option<&'static str>>,
}

type BuiltinCallback = extern "C" fn(
    name: *mut c_char,
    args: *mut *mut c_char,
    opts: *mut zsys::options,
    _arg: i32,
) -> i32;

impl ModuleHolder {
    const fn empty() -> Self {
        Self {
            module: parking_lot::const_mutex(None),
            panicked: AtomicBool::new(false),
            name: parking_lot::const_mutex(None),
        }
    }

    pub fn set_name(&self, name: &'static str) {
        let _ = self.name.lock().insert(name);
    }

    pub fn set_mod(&self, mut module: Module, builtin_callback: BuiltinCallback) {
        for x in module.features.get_binaries() {
            x.handlerfunc = Some(builtin_callback)
        }
        *self.module.lock() = Some(module);
    }

    fn get_mod<'a>(&'a self) -> parking_lot::MappedMutexGuard<'a, Module> {
        parking_lot::MutexGuard::map(self.module.lock(), |opt| {
            opt.as_mut().expect("No module set")
        })
    }

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

    pub fn builtin_callback(
        &self,
        name: *mut c_char,
        args: *mut *mut c_char,
        opts: *mut zsys::options,
        _: i32,
    ) -> i32 {
        let module_holder = AssertUnwindSafe(self);
        handle_panic(|| {
            let args = unsafe { crate::CStrArray::from_raw(args.cast()) };
            let name = unsafe { CStr::from_ptr(name) };
            let opts = unsafe { Opts::from_raw(opts) };

            let Module {
                bintable,
                user_data,
                ..
            } = &mut *module_holder.get_mod();
            let Some(bin) = bintable.get_mut(name) else {
                return 3;
            };
            match bin(&mut **user_data, name, args, opts) {
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
}

// This struct is neither of them, but since it isn't exposed to user code
// and it isn't given to any threads, this should be safe.
unsafe impl Sync for ModuleHolder {}
unsafe impl Send for ModuleHolder {}

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
    pub use zsh_sys as zsys;
}

#[macro_export]
/// Exports a `setup` function to be called when the module needs to be set up.
/// You need to specify your module's loadable name
macro_rules! export_module {
    ($module_name:ident, $setupfn:ident) => {
        mod _zsh_private_glue {
            use $crate::export_module::{ffi, MODULE, handle_panic, handle_maybe_error};

            static MOD_NAME: &'static str = stringify!($module_name);

            extern "C" fn handle_builtin(
                name: *mut c_char,
                args: *mut *mut c_char,
                opts: *mut ffi::zsys::options,
                _arg: i32,
            ) -> i32 {
                MODULE.builtin_callback(name, args, opts, _arg)
            }

            #[no_mangle]
            extern "C" fn setup_(_: ffi::Module) -> i32 {
                handle_panic(|| {
                    let res = super::$setupfn().map(|module| {
                        MODULE.set_name(MOD_NAME);
                        MODULE.set_mod(module, handle_builtin)
                    }
                    );
                    handle_maybe_error(res)
                })
                .unwrap_or(65)
            }

            use ::std::ffi::{ c_char, c_int };
            $crate::export_module!(@fn boot_(module: ffi::Module));
            $crate::export_module!(@fn features_(module: ffi::Module, features_ptr: *mut *mut *mut c_char));
            $crate::export_module!(@fn enables_(module: ffi::Module, enables_ptr: *mut *mut c_int));
            $crate::export_module!(@fn cleanup_(module: ffi::Module));
            $crate::export_module!(@fn finish_(module: ffi::Module) );
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
        let mut module = MODULE.get_mod();
        unsafe { *features_ptr = zsys::featuresarray(mod_, &mut *module.features) };
        0
    }
);

mod_fn!(
    fn enables_(mod_, enables_ptr: *mut *mut c_int) {
        let mut module = MODULE.get_mod();
        unsafe {
            zsys::handlefeatures(mod_, &mut *module.features, enables_ptr)
        }
    }
);

// Called when cleaning the module up.
mod_fn!(
    fn cleanup_(_mod) {
        let mut module = MODULE.get_mod();
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
