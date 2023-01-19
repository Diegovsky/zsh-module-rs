//! # Zsh Module
//! This is a high level crate that allows you to define your own zsh module.
//!
//! ## Getting started
//! To get started, first, you need to create library, not an executable. Then, change your crate
//! type to `"cdylib"` on your `Cargo.toml`:
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! ## Boilerplate
//! On your `lib.rs`, you need to put a [`export_module!`] macro call, alongside a `setup` function
//! (can be called whatever you want):
//! ```rust
//! use zsh_module::{ Module, ModuleBuilder }
//!
//! zsh_module::export_module!(setup);
//!
//! fn setup() -> Result<Module, ()> {
//!    todo!()
//! }
//! ```
//!
//! ## Defining [`Actions`]
//! The main point part of crating a module is implementing [`Actions`]. Here's an example module:
//! ```rust
//! use zsh_module::{ Module, ModuleBuilder, Actions, Result }
//!
//! zsh_module::export_module!(setup);
//!
//! struct Greeter;
//!
//! impl Actions for Greeter {
//!     fn boot(mut &self) -> Result<()> {
//!         println!("Hello, everyone!");
//!         Ok(())
//!     }
//!     fn cleanup(&mut self) -> Result<()> {
//!         println!("Bye, everyone!");
//!         Ok(())
//!     }
//! }
//!
//! fn setup() -> Result<Module> {
//!     let module = ModuleBuilder::new(Greeter)
//!         .build();
//!     Ok(module)
//! }
//! ```
//!
//! ## Installing
//! When your module is ready, copy your shared library to your distribution's zsh module folder
//! and name it whatever you want, the only requirement is that it ends with your platforms's
//! dynamic loadable library extension.
//!
//! On my machine, the zsh module folder is `/usr/lib/zsh/<zsh-version>/zsh/`.
//!
//! If everything went fine, you can load it in zsh using the following command:
//! ```zsh
//! zmodload zsh/<module-name>
//! ```
//!
//! That is it!

#![feature(trait_alias)]
use std::{
    borrow::Borrow,
    collections::HashMap,
    ffi::{CStr, CString},
    marker::PhantomData,
    pin::Pin,
};

use downcast_rs::Downcast;

use features::Features;
#[doc(hidden)]
pub use zsh_sys as zsys;

mod features;
pub mod log;
/// This crate's error type.
pub type Error = Box<dyn std::error::Error>;

/// `Result<T, Error>`
pub type Result<T> = std::result::Result<T, Error>;

trait AnyCmd = FnMut(&mut (dyn Actions + 'static), &str, &[&str]) -> Result<()>;

/// This trait corresponds to the function signature of a zsh builtin command handler.
///
/// See [`ModuleBuilder::builtin`] for how to register a command.
pub trait Cmd<A: Actions> = 'static + FnMut(&mut A, &str, &[&str]) -> Result<()>;

/// This trait allows for defining behaviour to be enacted on parts of the zsh module lifecycle.
///
/// Refer to each method for a description of when it is called
pub trait Actions: Downcast {
    /// This method is called right after `setup()`.
    ///
    /// If this returns [`Err`], loading is cancelled and [`Self::cleanup`] is not called.
    fn boot(&mut self) -> Result<()>;
    /// This method is only called if the module was successfully loaded.
    ///
    /// An [`Err`] result means the module is not ready to unload yet. However, zsh will ignore
    /// this if it is exiting.
    fn cleanup(&mut self) -> Result<()>;
}

downcast_rs::impl_downcast!(Actions);

#[derive(PartialEq, Hash, Debug, Eq)]
#[doc(hidden)]
pub struct PinnedCStr(Pin<Box<CStr>>);

impl PinnedCStr {
    fn new(string: &str) -> Self {
        Self(Box::into_pin(
            CString::new(string)
                .expect("Strings should not contain null byte!")
                .into_boxed_c_str(),
        ))
    }
    #[allow(dead_code)]
    fn ptr(&self) -> *const i8 {
        self.0.as_ptr()
    }
    fn ptr_mut(&mut self) -> *mut i8 {
        unsafe { std::mem::transmute(self.0.as_ptr()) }
    }
}

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

/// Properties of a zsh builtin command
///
/// Any chages will reflect on the behaviour of the builtin
pub struct Builtin<'a, C, A>
where
    C: Cmd<A>,
{
    minargs: i32,
    maxargs: i32,
    flags: Option<&'static str>,
    cb: C,
    name: &'a str,
    _phantom: PhantomData<A>,
}

impl<'a, A, C> Builtin<'a, C, A>
where
    A: Actions,
    C: Cmd<A> + 'static,
{
    /// Creates a command builtin.
    ///
    /// By default, the builtin can take any amount of arguments (minargs and maxargs are 0 and
    /// None, respectively) and no flags.
    pub fn new(name: &'static str, cb: C) -> Self {
        Self {
            minargs: 0,
            maxargs: -1,
            flags: None,
            name,
            cb,
            _phantom: PhantomData,
        }
    }
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
}

type Bintable = HashMap<PinnedCStr, Box<dyn AnyCmd>>;

/// Allows you to build a [`Module`]
pub struct ModuleBuilder<A> {
    actions: A,
    binaries: Vec<zsys::builtin>,
    bintable: Bintable,
    strings: Vec<PinnedCStr>,
}

impl<A> ModuleBuilder<A>
where
    A: Actions + 'static,
{
    //! Creates an empty [`Self`] with options ready for configuration.
    pub fn new(actions: A) -> Self {
        Self {
            actions,
            binaries: vec![],
            bintable: HashMap::new(),
            strings: Vec::with_capacity(16),
        }
    }
    fn hold_string(&mut self, value: &str) -> *mut i8 {
        let mut value = PinnedCStr::new(value);
        let ptr = value.ptr_mut();
        self.strings.push(value);
        ptr
    }
    /// Registers a new builtin command
    pub fn builtin<C: Cmd<A>>(self, builtin: Builtin<C, A>) -> Self {
        let mut cb = builtin.cb;
        let closure: Box<dyn AnyCmd> =
            Box::new(move |actions: &mut (dyn Actions + 'static), name, args| {
                cb(actions.downcast_mut::<A>().unwrap(), name, args)
            });
        self.add_builtin(
            builtin.name,
            builtin.minargs,
            builtin.maxargs,
            builtin.flags,
            closure,
        )
    }
    fn add_builtin(
        mut self,
        name: &str,
        minargs: i32,
        maxargs: i32,
        flags: Option<&'static str>,
        cb: Box<dyn AnyCmd + 'static>,
    ) -> Self {
        let mut name = PinnedCStr::new(name);
        let flags = match flags {
            Some(flags) => self.hold_string(flags),
            None => std::ptr::null_mut(),
        };
        let cb = Box::new(cb);
        let raw = zsys::builtin {
            node: zsys::hashnode {
                next: std::ptr::null_mut(),
                nam: name.ptr_mut(),
                // !TODO: add flags param
                flags: 0,
            },
            // The handler function will be set later by the zsh module glue
            handlerfunc: None,
            minargs,
            maxargs,
            funcid: 0,
            optstr: flags,
            defopts: std::ptr::null_mut(),
        };
        self.binaries.push(raw);
        self.bintable.insert(name, cb);
        self
    }
    /// Creates a new module, ready to be used.
    pub fn build(self) -> Module {
        Module::new(self)
    }
}

#[allow(dead_code)]
/// A zsh module. You must build it using [`ModuleBuilder`]
pub struct Module {
    actions: Box<dyn Actions>,
    features: Features,
    bintable: Bintable,
    strings: Vec<PinnedCStr>,
}

impl Module {
    fn new<A: Actions + 'static>(desc: ModuleBuilder<A>) -> Self {
        let features = Features::empty().binaries(desc.binaries.into_boxed_slice());
        Self {
            actions: Box::new(desc.actions),
            features,
            bintable: desc.bintable,
            strings: desc.strings,
        }
    }
}

#[cfg(feature = "export_module")]
mod export_module;
