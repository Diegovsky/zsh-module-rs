#![doc=include_str!("../../README.md")]

#![feature(trait_alias)]
use std::{ffi::{c_char, CStr, CString}, collections::HashMap, pin::Pin, borrow::Borrow};

#[doc(hidden)]
pub use zsh_sys as zsys;

pub mod log;

struct Features {
    pub raw: zsys::features
}

impl Features {
    pub fn empty() -> Self {
        unsafe {
            std::mem::MaybeUninit::zeroed().assume_init()
        }
    }
    pub fn binaries(&mut self, binaries: Box<[zsys::builtin]>) -> &mut Self {
        #[allow(unused_unsafe)]
        unsafe {
            let mem = Box::leak(binaries);
            self.raw.bn_list = mem.as_mut_ptr();
            self.raw.bn_size = mem.len() as i32;
        };
        self
    }
    pub fn conddef(&mut self, defs: Box<[zsys::conddef]>) -> &mut Self {
        #[allow(unused_unsafe)]
        unsafe {
            let mem = Box::leak(defs);
            self.raw.cd_list = mem.as_mut_ptr();
            self.raw.cd_size = mem.len() as i32;
        };
        self
    }
    pub fn mathfuncs(&mut self, funcs: Box<[zsys::mathfunc]>) -> &mut Self {
        #[allow(unused_unsafe)]
        unsafe {
            let mem = Box::leak(funcs);
            self.raw.mf_list = mem.as_mut_ptr();
            self.raw.mf_size = mem.len() as i32;
        };
        self
    }
    pub fn paramdefs(&mut self, binaries: Box<[zsys::paramdef]>) -> &mut Self {
        #[allow(unused_unsafe)]
        unsafe {
            let mem = Box::leak(binaries);
            self.raw.pd_list = mem.as_mut_ptr();
            self.raw.pd_size = mem.len() as i32;
        };
        self
    }
}

#[doc(hidden)]
pub unsafe fn strings_from_ptr<'a>(mut ptr: *const *const c_char) -> Vec<&'a str> {
    let mut vec = Vec::with_capacity(2);
    loop {
        if *ptr == std::ptr::null() {
            break vec
        }
        vec.push(CStr::from_ptr(*ptr).to_str().expect("Failed to parse arg"));
        ptr = ptr.add(1);
    }
}

/// This trait corresponds to the function signature of a zsh builtin command handler.
///
/// See [`ModuleBuilder::builtin`] for how to register a command.
pub trait Cmd = FnMut(&dyn Actions, &str, &[&str])->Result<()>;


/// This crate's error type.
type Error = Box<dyn std::error::Error + Send>;

pub type Result<T> = std::result::Result<T, Error>;

/// This trait allows for defining behaviour to be enacted on parts of the zsh module lifecycle.
///
/// Refer to each method for a description of when it is called
pub trait Actions {
    /// This method is called right after `setup()`.
    ///
    /// If this returns [`Err`], loading is cancelled and [`Self::cleanup`] is not called.
    fn boot (&mut self) -> Result<()>;
    /// This method is only called if the module was successfully loaded.
    ///
    /// An [`Err`] result means the module is not ready to unload yet. However, zsh will ignore
    /// this if it is exiting.
    fn cleanup (&mut self) -> Result<()>;
}

#[derive(PartialEq, Hash, Debug, Eq)]
#[doc(hidden)]
pub struct PinnedCStr(Pin<Box<CStr>>);

impl std::ops::Deref for PinnedCStr {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Borrow<CStr> for PinnedCStr {
    fn borrow(&self) -> &CStr {
       &*self.0
    }
}

type Bintable = HashMap<PinnedCStr, Box<dyn Cmd>>;

/// Allows you to build a [`Module`]
pub struct ModuleBuilder {
    binaries: Vec<zsys::builtin>,
    bintable: Bintable,
}

extern "C"{ fn zsh_private_callback(name: *mut i8, args: *mut *mut i8, opts: *mut zsys::options, _: i32) -> i32; }

/// Properties of a zsh builtin command
///
/// Any chages will reflect on the behaviour of the builtin
pub struct BuiltinBuilder {
    builder: ModuleBuilder,
    minargs: i32,
    maxargs: i32,
    flags: Option<&'static str>,
}

impl BuiltinBuilder {
    fn new(builder: ModuleBuilder) -> Self { Self { builder, minargs: 0, maxargs: -1, flags: None } }
    /// Sets the minimum amount of arguments allowed by the builtin
    pub fn minargs(mut self, value: i32) -> Self {
        self.minargs = value; 
        self
    }
    /// Sets the maximum amount of arguments allowed by the builtin
    pub fn maxargs(mut self, value: Option<u32>) -> Self {
        self.maxargs = value.map(|i| i as i32).unwrap_or(-1);
        self
    }
    /// Sets flags recognized by the builtin
    pub fn flags(mut self, value: &'static str) -> Self {
        self.flags = Some(value);
        self
    }
    /// Registers the builtin.
    ///
    /// By default, the builtin can take any amount of arguments (minargs and maxargs are 0 and
    /// None, respectively) and no flags.
    pub fn build(self, name: &'static str, cb: impl Cmd + 'static) -> ModuleBuilder {
        self.builder.add_builtin(name, self.minargs, self.maxargs, self.flags, cb)
    }
}

impl ModuleBuilder {
    //! Creates an empty [`Self`] with options ready for configuration.
    pub fn new() -> Self {
        Self { 
            binaries: vec![],
            bintable: HashMap::new(),
        }
    }
    /// Registers a new builtin command
    pub fn builtin(self) -> BuiltinBuilder {
        BuiltinBuilder::new(self)
    }
    fn add_builtin(mut self, name: &'static str, minargs: i32, maxargs: i32, flags: Option<&'static str>, cb: impl Cmd + 'static) -> Self {
        let name = PinnedCStr(Box::into_pin(CString::new(name).unwrap().into_boxed_c_str()));
        let cb = Box::new(cb);
        let raw = zsys::builtin {
            node: zsys::hashnode{
                next: std::ptr::null_mut(),
                nam: unsafe { std::mem::transmute(name.as_ptr()) },
                // !TODO: add flags param
                flags: 0,
            },
            handlerfunc: Some(zsh_private_callback),
            minargs,
            maxargs,
            funcid: 0,
            optstr: flags.map(|flags| unsafe { std::mem::transmute(flags.as_ptr()) }).unwrap_or(std::ptr::null_mut()),
            defopts: std::ptr::null_mut(),
        };
        self.binaries.push(raw);
        self.bintable.insert(name, cb);
        self
    }
    /// Creates a new module, ready to be used.
    pub fn build(self, actions: impl Actions + 'static) -> Module {
        Module::new(actions, self)
    }
}

#[allow(dead_code)]
/// A zsh module. You must build it using [`ModuleBuilder`]
pub struct Module {
    #[doc(hidden)]
    pub actions: Box<dyn Actions>,
    #[doc(hidden)]
    pub features: zsys::features,
    #[doc(hidden)]
    pub bintable: Bintable,
}

impl Module {
    fn new(actions: impl Actions + 'static, desc: ModuleBuilder) -> Self {
        let features = Features::empty()
            .binaries(desc.binaries.into_boxed_slice())
            .raw;
        Self {
            actions: Box::new(actions),
            features,
            bintable: desc.bintable,
        }
    }


}

#[doc(hidden)]
#[macro_export]
macro_rules! mod_fn {
    (fn $name:ident($mod:ident $(,$arg:ident : $type:ty)*) try $block:expr) => {
        mod_fn!(
            fn $name($mod $(,$arg : $type)*) {
                match $block {
                    Ok(()) =>  0,
                    Err(e) => { 1 },
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

/// This macro defines hooks to the zsh C functions, therefore, you **must** put this in the same
/// file as your `setup` function.
#[macro_export]
macro_rules! impl_hooks {
    () => {

#[doc(hidden)]
mod zsh_private_hooks {

use $crate::mod_fn;
use std::ffi::{c_char, c_int, CStr};
use $crate::zsys;
use $crate::*;

#[no_mangle]
unsafe extern "C" fn zsh_private_callback(name: *mut i8, args: *mut *mut i8, _opts: *mut zsys::options, _: i32) -> i32 {
    let args = unsafe { $crate::strings_from_ptr(std::mem::transmute(args)) };
    let name = unsafe { CStr::from_ptr(name) };
    let mod_ = get_mod();
    let bin = mod_.bintable.get_mut(name).expect("Failed to find binary name");
    match bin(&*mod_.actions, name.to_str().expect("Failed to parse binary name"), &args) {
        Ok(()) => 0,
        Err(e) => { $crate::error!("Error: {}", e) ;1 },
    }
}


static mut MODULE: Option<Module> = None;

fn get_optmod() -> &'static mut Option<Module> {
    unsafe { &mut MODULE }
}

fn get_mod() -> &'static mut Module {
    get_optmod().as_mut().expect("Module not set") 
}

mod_fn!(
    fn setup_(_mod) {
        let module = match super::setup() {
            Ok(module) => module,
            Err(e) => {
                $crate::error!("Failed to setup module: {}", e);
                return 1
            }
        };
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
        unsafe { *features_ptr = zsys::featuresarray(mod_, &mut module.features) };
        0
    }
);

mod_fn!(
    fn enables_(mod_, enables_ptr: *mut *mut c_int) {
        let module = get_mod();
        unsafe {
            zsys::handlefeatures(mod_, &mut module.features, enables_ptr)
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
        Ok::<(), ()>(())
    }
);
} // mod zsh_private_hooks
}; // impl_hooks
}
