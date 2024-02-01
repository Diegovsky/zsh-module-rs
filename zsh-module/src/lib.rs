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
//! ```no_run
//! use zsh_module::{ Module, ModuleBuilder };
//!
//! zsh_module::export_module!(my_module, setup);
//!
//! fn setup() -> Result<Module, Box<dyn std::error::Error>> {
//!    todo!()
//! }
//! ```
//! ## The `setup` function
//! A proper `setup` function must return a [`Result<Module, E>`] where `E` implements
//! [`Error`][std::error::Error]. E.g:
//! ```ignore
//! fn setup() -> Result<Module, Box<dyn std::error::Error>> { .. }
//!
//! fn setup() -> Result<Module, anyhow::Error> { .. }
//!
//! fn setup() -> Result<Module, std::io::Error> { .. }
//! ```
//!
//! ## Storing User Data
//! You can store user data inside a module and have it accessible from any callbacks.
//! Here's an example module, located at  that defines a new `greet` builtin command:
//! ```no_run
//! use zsh_module::{Builtin, MaybeZError, Module, ModuleBuilder, Opts};
//!
//! // Notice how this module gets installed as `rgreeter`
//! zsh_module::export_module!(rgreeter, setup);
//!
//! struct Greeter;
//!
//! impl Greeter {
//!     fn greet_cmd(&mut self, _name: &str, _args: &[&str], _opts: Opts) -> MaybeZError {
//!         println!("Hello, world!");
//!         Ok(())
//!     }
//! }
//!
//! fn setup() -> Result<Module, Box<dyn std::error::Error>> {
//!     let module = ModuleBuilder::new(Greeter)
//!         .builtin(Greeter::greet_cmd, Builtin::new("greet"))
//!         .build();
//!     Ok(module)
//! }
//! ```
//!
//! ## Installing
//! When your module is ready, copy your shared library to any folder in your `$module_path`
//! and name it whatever you want, the only requirement is that it ends with your platforms's
//! dynamic loadable library extension.
//!
//! To add a folder to your `$module_path`, add the following code to your `.zshrc`:
//!
//! ```sh no_run
//! typeset -aUg module_path
//! module_path+=($HOME/.zsh/modules)
//! ```
//!
//! For development, you can consider symlinking the library into that folder in your `$module_path`.
//!
//! ```sh no_run
//! ln -s "$PWD/target/debug/libmodule.so" "$HOME/.zsh/modules/module.so"
//! ```
//!
//! If everything went fine, you can load it in zsh using the following command:
//! ```sh no_run
//! zmodload <module-name>
//! ```
//!
//! That is it!

// My goal is to eliminate the need for this nightly feature.
#![feature(trait_alias)]

use std::{
    any::Any,
    borrow::Cow,
    collections::HashMap,
    ffi::{c_char, CStr, CString, OsStr},
    fmt, io,
    os::unix::ffi::OsStrExt,
    path::*,
};

use features::Features;

pub use options::Opts;
use zsh_sys as zsys;

mod features;
mod hashtable;
pub mod log;
mod options;
pub mod terminal;
pub mod types;
// pub mod variable;
pub mod zsh;

pub use crate::types::{
    cstring::{to_cstr, ToCString},
    error::*,
};
pub use hashtable::HashTable;

// TODO: Rewrite this to compile in stable rust
trait AnyCmd = Cmd<dyn Any, ZError>;

/// This trait corresponds to the function signature of a zsh builtin command handler.
///
/// # Generics
///  - `A` is your User Data. For more info, read [`Storing User Data`]
///  - `E` is anything that can be turned into a [`Box`]ed error.
///
/// # Example
/// ```
///     fn hello_cmd(data: &mut (), _cmd_name: &str, _args: &[&str], opts: zsh_module::Opts) -> zsh_module::MaybeZError {
///         println!("Hello, world!");
///         let some_result = some_function(some_opts);
///         // In this example, the eerror from `some_result` does not fit nicely into a Zerror
///         if let Err(e) = some_result {
///             return Err(ZError::Runtime(e.to_string()));
///         }
///         Ok(())
///     }
/// ```
///
/// # See Also
/// See [`ModuleBuilder::builtin`] for how to register a command.
pub trait Cmd<A: Any + ?Sized, E: Into<ZError>> =
    'static + FnMut(&mut A, &str, &[&str], Opts) -> Result<(), E>;

// TODO: Rewrite it like this to compile in stable rust
// pub trait Cmd<A: Any + ?Sized, E: Into<ZError>>:
//     'static + FnMut(&mut A, &str, &[&str], Opts) -> Result<(), E>
// {}

/// Properties of a zsh builtin command.
///
/// Any chages will reflect on the behaviour of the builtin
pub struct Builtin {
    minargs: i32,
    maxargs: i32,
    flags: Option<CString>,
    name: CString,
}

impl Builtin {
    /// Creates a command builtin.
    ///
    /// By default, the builtin can take any amount of arguments (minargs and maxargs are 0 and
    /// [`None`], respectively) and no flags.
    pub fn new(name: &str) -> Self {
        Self {
            minargs: 0,
            maxargs: -1,
            flags: None,
            name: to_cstr(name),
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
    pub fn flags(mut self, value: &str) -> Self {
        self.flags = Some(to_cstr(value));
        self
    }
}

type Bintable = HashMap<Box<CStr>, Box<dyn AnyCmd>>;

/// Allows you to build a [`Module`]
pub struct ModuleBuilder<A> {
    user_data: A,
    binaries: Vec<zsys::builtin>,
    bintable: Bintable,
    strings: Vec<Box<CStr>>,
    // paramtab_hook: i,
}

impl<A> ModuleBuilder<A>
where
    A: Any + 'static,
{
    //! Creates an empty [`Self`] with options ready for configuration.
    pub fn new(user_data: A) -> Self {
        Self {
            user_data,
            binaries: vec![],
            bintable: HashMap::new(),
            strings: Vec::with_capacity(8),
        }
    }
    /// Registers a new builtin command
    ///
    /// TODO: This requires the trait alias thing. Idk how to rewrite it to use the stable rust {} pattern.
    pub fn builtin<E, C>(self, mut cb: C, builtin: Builtin) -> Self
    where
        E: Into<ZError>,
        C: Cmd<A, E>,
    {
        let closure: Box<dyn AnyCmd> = Box::new(
            move |data: &mut (dyn Any + 'static), name, args, opts| -> Result<(), ZError> {
                cb(data.downcast_mut::<A>().unwrap(), name, args, opts).map_err(E::into)
            },
        );
        self.add_builtin(
            builtin.name,
            builtin.minargs,
            builtin.maxargs,
            builtin.flags,
            closure,
        )
    }
    fn hold_cstring(&mut self, value: impl Into<Vec<u8>>) -> *mut i8 {
        let value = to_cstr(value).into_boxed_c_str();
        let ptr = value.as_ptr();
        self.strings.push(value);
        ptr as *mut _
    }
    fn add_builtin(
        mut self,
        name: CString,
        minargs: i32,
        maxargs: i32,
        options: Option<CString>,
        cb: Box<dyn AnyCmd + 'static>,
    ) -> Self {
        let name = name.into_boxed_c_str();
        let flags = match options {
            Some(flags) => self.hold_cstring(flags),
            None => std::ptr::null_mut(),
        };
        let raw = zsys::builtin {
            node: zsys::hashnode {
                next: std::ptr::null_mut(),
                nam: name.as_ptr() as *mut _,
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

/// Hooks into the Zsh module system and connects it to your `User Data`.
pub struct Module {
    user_data: Box<dyn Any>,
    features: Features,
    bintable: Bintable,
    #[allow(dead_code)]
    strings: Vec<Box<CStr>>,
    name: Option<&'static str>,
}

impl Module {
    fn new<A: Any + 'static>(desc: ModuleBuilder<A>) -> Self {
        let features = Features::empty().binaries(desc.binaries.into());
        Self {
            user_data: Box::new(desc.user_data),
            features,
            bintable: desc.bintable,
            strings: desc.strings,
            name: None,
        }
    }
}

#[cfg(feature = "export_module")]
#[doc(hidden)]
pub mod export_module;
