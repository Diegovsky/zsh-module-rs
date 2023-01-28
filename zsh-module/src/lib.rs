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
//! zsh_module::export_module!(setup);
//!
//! fn setup() -> Result<Module, Box<dyn std::error::Error>> {
//!    todo!()
//! }
//! ```
//! ## The `setup` function
//! A proper `setup` function must return a [`Result<Module, E>`] where `E` implements
//! [`Error`][std::error::Error]. E.g:
//! ```no_run
//! fn setup() -> Result<Module, Box<dyn std::error::Error>> { .. }
//!
//! fn setup() -> Result<Module, anyhow::Error> { .. }
//!
//! fn setup() -> Result<Module, std::io::Error> { .. }
//! ```
//!
//! ## Storing User Data
//! You can store user data inside a module and have it accessible from any callbacks.
//! Here's an example module that defines a new `greet` builtin command:
//! ```
//! use zsh_module::{ Module, ModuleBuilder, MaybeError, Opts, Builtin };
//!
//! zsh_module::export_module!(setup);
//!
//! struct Greeter;
//!
//! impl Greeter {
//!     fn greet_cmd(&mut self, name: &str, args: &[&str], opts: Opts) -> MaybeError {
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
//! When your module is ready, copy your shared library to your distribution's zsh module folder
//! and name it whatever you want, the only requirement is that it ends with your platforms's
//! dynamic loadable library extension.
//!
//! On my machine, the zsh module folder is `/usr/lib/zsh/<zsh-version>/zsh/`.
//!
//! If everything went fine, you can load it in zsh using the following command:
//! ```sh no_run
//! zmodload zsh/<module-name>
//! ```
//!
//! That is it!

#![feature(trait_alias)]
use std::{
    any::Any,
    collections::HashMap,
    error::Error,
    ffi::{CStr, CString, c_char}, borrow::Cow
};

use features::Features;

pub use options::Opts;
use zsh_sys as zsys;

mod features;
mod hashtable;
pub mod log;
mod options;
pub mod zsh;

pub use hashtable::HashTable;

/// A box error type for easier error handling.
pub type AnyError = Box<dyn Error>;

/// Represents the possibility of an error `E`.
/// It is basically a [`Result`] that only cares for its [`Err`] variant.
///
/// # Generics
/// You can (and should) replace the default error type `E` with your own [`Error`].
pub type MaybeError<E = AnyError> = Result<(), E>;

trait AnyCmd = Cmd<dyn Any, AnyError>;

/// This trait corresponds to the function signature of a zsh builtin command handler.
///
/// # Generics
///  - `A` is your User Data. For more info, read [`Storing User Data`]
///  - `E` is anything that can be turned into a [`Box`]ed error.
///
/// # Example
/// ```
///     fn hello_cmd(data: &mut (), _cmd_name: &str, _args: &[&str], opts: zsh_module::Opts) -> zsh_module::MaybeError {
///         println!("Hello, world!");
///         Ok(())
///     }
/// ```
///
/// # See Also
/// See [`ModuleBuilder::builtin`] for how to register a command.
pub trait Cmd<A: Any + ?Sized, E: Into<AnyError>> =
    'static + FnMut(&mut A, &str, &[&str], Opts) -> MaybeError<E>;

pub(crate) fn to_cstr(string: impl Into<Vec<u8>>) -> CString {
    CString::new(string).expect("Strings should not contain a null byte!")
}

/// Represents any type that can be represented as a C String. You shouldn't
/// need to implement this yourself as the most commonly used `string`-y types
/// already have this implemented.
///
/// # Examples
/// ```
/// use std::ffi::{CString, CStr};
/// use std::borrow::Cow;
///
/// use zsh_module::ToCString;
///
/// let cstr = CStr::from_bytes_with_nul(b"Hello, world!\0").unwrap();
/// let cstring = CString::new("Hello, world!").unwrap();
///
/// assert!(matches!(cstr.into_cstr(), Cow::Borrowed(data) if data == cstr));
/// 
/// let string = "Hello, world!";
/// assert!(matches!(ToCString::into_cstr(string), Cow::Owned(cstring)));
/// ```
pub trait ToCString {
    fn into_cstr<'a>(self) -> Cow<'a, CStr> where Self: 'a;
}


macro_rules! impl_tocstring {
    ($($type:ty),*) => {
        $(impl ToCString for $type {
            fn into_cstr<'a>(self) -> Cow<'a, CStr> where Self: 'a {
                Cow::Owned(to_cstr(self))
            }
        })*
    };
}

impl_tocstring!(Vec<u8>, &[u8], &str, String);

impl ToCString for &CStr {
    fn into_cstr<'a>(self) -> Cow<'a, CStr> where Self: 'a {
        Cow::Borrowed(self)
    }
}

impl ToCString for CString {
    fn into_cstr<'a>(self) -> Cow<'a, CStr> {
        Cow::Owned(self)
    }
}

impl ToCString for *const c_char {
    fn into_cstr<'a>(self) -> Cow<'a, CStr> {
        Cow::Borrowed(unsafe { CStr::from_ptr(self) })
    }
}

impl ToCString for *mut c_char {
    fn into_cstr<'a>(self) -> Cow<'a, CStr> {
        Cow::Borrowed(unsafe { CStr::from_ptr(self) })
    }
}

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
    pub fn builtin<E, C>(self, mut cb: C, builtin: Builtin) -> Self
    where
        E: Into<Box<dyn Error>>,
        C: Cmd<A, E>,
    {
        let closure: Box<dyn AnyCmd> = Box::new(
            move |data: &mut (dyn Any + 'static), name, args, opts| -> MaybeError<AnyError> {
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
}

impl Module {
    fn new<A: Any + 'static>(desc: ModuleBuilder<A>) -> Self {
        let features = Features::empty().binaries(desc.binaries.into());
        Self {
            user_data: Box::new(desc.user_data),
            features,
            bintable: desc.bintable,
            strings: desc.strings,
        }
    }
}

#[cfg(feature = "export_module")]
mod export_module;
