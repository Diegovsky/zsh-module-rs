use crate::ZError;
use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, CString},
    fmt, iter,
    sync::{
        mpsc::{SendError, Sender},
        Arc,
    },
};

/// The type we're using for the name of the variable, as well as hashmap keys. May change in the future if need be.
pub type VariableKey = String;

/// The type we're using for scalar (string) variables
pub type Scalar = String;

/// The type we're sending to the internal mpsc channel
pub type MpscVarType = (VariableKey, VarType);

/// WIP definition of a variable
/// ```
/// VariableBuilder::new("PAGER").build()?;
/// ```
///
/// TODO: Integrate with zsh
///
/// TODO: This is an owned value right now. Its existence should be tied directly to the referenced variable in zsh.
#[derive(Debug)]
pub struct Variable {
    name: VariableKey,
    // values may or may not be initialized. That's like running `typeset -a array` with no values.
    value: Option<VarType>,
    /// All the special properties of this variable
    flags: HashSet<TypeFlags>,
    // TODO: This is an idea I had for thread-safe variable access. It may or may not be practical.
    // mpsc_channel: Arc<Sender<MpscVarType>>,
}
impl Variable {
    /// This variable's name. Goes out of scope when the variable is dropped.
    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }
    /// This variable's value Goes out of scope when the variable is dropped.
    pub fn value<'a>(&'a self) -> Option<&'a VarType> {
        if let Some(v) = &self.value {
            Some(v)
        } else {
            None
        }
    }
    /// Change the types of this variable at runtime.
    ///
    /// This is equivalent to running `typeset <flag> variable` when the variable is already defined in the shell.
    pub fn typeset<I>(&mut self, flags: I) -> Result<&mut Self, VarError>
    where
        I: IntoIterator<Item = TypeFlags>,
    {
        let flags = flags.into_iter();
        todo!()
    }
    /// Update the value of this variable. This calls internal zsh functions
    ///
    /// TODO: Implement
    pub fn set(&mut self, value: VarType) -> Result<(), VarError> {
        if self.flags.contains(&TypeFlags::ReadOnly) {
            return Err(VarError::ValueSet(VarIntrospectionError::NotPermitted));
        }
        // if let Err(e) = self.mpsc_channel.send((self.name, value)) {
        //     match e {
        //         SendError(_) => {
        //             return Err(Zerror::Custom(
        //                 "Could not send to internal mpsc channel".to_string(),
        //             ))
        //         }
        //     }
        // }
        todo!();
        Ok(())
    }
    /// Get the current value of the variable from the environment, saving it in the internal cache that you can access with the `value` method.
    ///
    /// TODO: Implement, this might be redundant. There would likely be a time-accessed-to-time-updated problem if it used a cache.
    pub fn refresh(&mut self) -> Result<(), VarError> {
        todo!()
    }
}
impl ZVariable for Variable {
    fn has_value(&self) -> bool {
        if let Some(v) = &self.value {
            v.has_value()
        } else {
            false
        }
    }
    fn length(&self) -> usize {
        if let Some(v) = &self.value {
            v.length()
        } else {
            0
        }
    }
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Primitive> + 'a> {
        if let Some(v) = &self.value {
            v.iter()
        } else {
            Box::new(iter::empty())
        }
    }
}

/// The special variable type flags
///
/// You can print most of the flags of a zsh variable using `echo ${(t)variable}`
///
/// For further information, read the manpage `man zshexpn` or go to
/// https://zsh.sourceforge.io/Doc/Release/Expansion.html#Parameter-Expansion
///
/// Example: The type of the `mapfile` array from `zsh/mapfile`
///
/// ```zsh
/// print ${(t)mapfile}
/// # association-hide-hideval-special
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeFlags {
    /// parameters that are local to the current function scope
    Local,
    /// for left justified parameters
    ///
    /// TODO: Learn what these are
    LeftJustified,
    /// for right justified parameters with leading blanks
    ///
    /// TODO: Learn what these are
    RightBlanks,
    /// Right-justified parameters with leading zeros
    ///
    /// TODO: Learn what these are
    RightZeros,
    /// parameters whose value is converted to all lower case when it is expanded
    ///
    /// TODO: Learn what these are
    Lower,
    /// parameters whose value is converted to all upper case when it is expanded
    ///
    /// TODO: Learn what these are
    Upper,
    /// Parameters that are read-only
    ReadOnly,
    /// Tagged parameters
    ///
    /// TODO: Learn what these are
    Tag,
    /// for parameters tied to another parameter in the manner of PATH (colon-separated list) and path (array),
    /// whether these are special parameters or user-defined with `typeset -T`
    ///
    /// This property contains the key of the variable that it is tied to.
    Tied(VariableKey),
    /// Parameters exported to subprocesses
    Export,
    /// Arrays that only keep the first occurrence of a duplicate value
    Unique,
    /// Parameters that are hidden
    Hide,
    /// Parameters that have the `hideval` attribute
    HideVal,
    /// Parameters that are defined by the shell
    Special,
}

/// Types of interaction with variables
#[derive(Debug, Default, Clone, Copy)]
pub enum InteractionType {
    Set,
    Unset,
    Listen,
    #[default]
    Get,
}

/// A builder for variables, similar to the zsh builtin `typeset`
///
/// Variable is meant to be a reference to a shell variable. This is meant to be owned and whatnot.
#[derive(Debug)]
pub struct VariableBuilder {
    pub name: VariableKey,
    pub value: Option<VarType>,
    pub flags: HashSet<TypeFlags>,
}
impl VariableBuilder {
    /// Create a new variable
    /// ```
    /// let var: Variable = VariableBuilder::new("HOME").build()?;
    /// assert_eq!(var.name(), "HOME");
    /// assert_eq!(var.value().unwrap(), "/home/antonio");
    /// ```
    pub fn new<S>(name: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            name: name.as_ref().to_string(),
            value: None,
            flags: HashSet::new(),
        }
    }
    /// Try to add a property to this variable. If it is already present, this will do nothing.
    ///
    /// For tied variables, there is a special key called [`TypeFlags::Tied`] that you can manually specify.
    /// If that variable does not exist already, it will be created and tied to this variable.
    ///
    /// If the variable exists already and is NOT tied to this variable, this will return an error.
    ///
    /// TODO: Find out if these are exclusive to certain variable types
    pub fn add_property(&mut self, prop: TypeFlags) -> Result<&mut Self, VarError> {
        // duplicate flag should only occur in this single case
        // TODO: This uses clone() because TypeFlags::Tied has a String internally. Find some way to resolve this.
        if let TypeFlags::Tied(key) = prop.clone() {
            if self.flags.contains(&prop) {
                return Err(VarTypesetError::TieDuplicateFlag(key).into());
            }

            // TODO: Check if they are actually tied or not
            if VariableBuilder::new(&key).build().is_err() {
                return Err(VarTypesetError::TieNotPermitted(key).into());
            }
        }

        // we chillin
        self.flags.insert(prop);
        Ok(self)
    }
    /// Set the preliminary value of this variable.
    pub fn value(&mut self, value: VarType) -> &mut Self {
        self.value = Some(value);
        self
    }
    /// Creates a [`Variable`] from this builder
    pub fn build(self) -> Result<Variable, VarError> {
        // might have set the preliminary cached value, so they probably didn't really want to query the value from the environment.
        let has_no_value = self.value.is_none();

        let mut out = Variable {
            name: self.name,
            value: self.value,
            flags: self.flags,
        };
        if has_no_value {
            out.refresh()?;
        }

        Ok(out)
    }
}

/// A variable primitive. All variables in zsh are typed.
#[derive(Debug)]
pub enum Primitive {
    Scalar(Scalar),
    Integer(isize),
    Float(f64),
}
impl ZVariable for Primitive {
    fn has_value(&self) -> bool {
        match self {
            Self::Scalar(s) => !s.is_empty(),
            _ => true, // integer and float types always have a default value of zero
        }
    }
    fn length(&self) -> usize {
        // TODO: detect multibyte and other shell options.
        match self {
            Self::Scalar(s) => s.chars().count(),
            Self::Integer(i) => (i.checked_ilog10().unwrap_or(0) + 1) as usize,
            Self::Float(f) => f.to_string().len(),
        }
    }
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Primitive> + 'a> {
        Box::new(iter::once(self))
    }
}
impl Default for Primitive {
    fn default() -> Self {
        // Returns a new String. This is what zsh does internally too.
        Self::Scalar(Scalar::default())
    }
}

/// The type of a variable, used internally
#[derive(Debug)]
pub enum VarType {
    Primitive(Primitive),
    Array(Vec<Primitive>),
    Association(HashMap<VariableKey, Primitive>),
}
impl ZVariable for VarType {
    fn has_value(&self) -> bool {
        match self {
            VarType::Primitive(p) => p.has_value(),
            VarType::Array(a) => !a.is_empty(),
            VarType::Association(h) => !h.is_empty(),
        }
    }
    fn length(&self) -> usize {
        match self {
            VarType::Primitive(p) => p.length(),
            VarType::Array(a) => a.len(),
            VarType::Association(h) => h.len(),
        }
    }
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Primitive> + 'a> {
        match self {
            VarType::Primitive(p) => p.iter(),
            VarType::Array(a) => Box::new(a.iter()),
            VarType::Association(h) => Box::new(h.values()),
        }
    }
}
impl Default for VarType {
    fn default() -> Self {
        Self::Primitive(Primitive::default())
    }
}

/// A trait for a variable's value. Supposed to be like a bunch of `test` commands
///
/// TODO: Add more commands
pub trait ZVariable {
    /// `[[ -n ${variable-} ]]`
    fn has_value(&self) -> bool;
    /// `$#variable`
    fn length(&self) -> usize;
    /// Iterate through the elements of a variable.
    ///
    /// If it is a scalar or integer, this just returns an iterator with a single element.
    ///
    /// If it is an array, this returns an iterator with the elements of the array.
    ///
    /// If it is an association, this returns an iterator with the VALUES of the association. Not the keys.
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Primitive> + 'a>;
}

/// Errors that could occur when trying to change a variable type at runtime
#[derive(Debug)]
pub enum VarTypesetError {
    ReadOnly,
    /// Should only occur in the rare case that the HashMap contains a duplicate enum key+value
    TieDuplicateFlag(VariableKey),
    /// Failure to tie variables
    TieNotPermitted(VariableKey),
    InvalidType,
    Disallowed,
}
impl fmt::Display for VarTypesetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "Variable is read-only"),
            Self::TieDuplicateFlag(t) => write!(f, "Duplicate type flag provided: {:?}", t),
            Self::TieNotPermitted(s) => write!(f, "Not permitted to tie variable: {}", s),
            Self::InvalidType => write!(f, "Invalid variable type"),
            Self::Disallowed => write!(f, "Variable is disallowed"),
        }
    }
}

/// Errors that could occur when interacting with the internal zsh variable
///
/// TODO: Discover more
#[derive(Debug)]
pub enum VarIntrospectionError {
    /// Variable is invalid in some weird way that wouldn't happen while using the shell language
    InvalidVariable,
    /// When the zsh internal paramtab == realparamtab check fails
    MisalignedParamTab,
    /// It is read-only or something
    NotPermitted,
    /// The variable doesn't exist
    Nonexistent,
}
impl fmt::Display for VarIntrospectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidVariable => write!(f, "Invalid variable"),
            Self::MisalignedParamTab => write!(f, "Misaligned paramtab"),
            Self::NotPermitted => write!(f, "Not permitted"),
            Self::Nonexistent => write!(f, "Variable doesn't exist"),
        }
    }
}

/// An error related to zsh variables
#[derive(Debug)]
pub enum VarError {
    /// An invalid assignment error that could occur when trying to change invalid variable options (such as setting a scalar to an array)
    Typeset(VarTypesetError),
    /// An error that could occur when trying to set the value of a variable
    ValueSet(VarIntrospectionError),
    /// An error that could occur when trying to unset a variable
    ValueUnset(VarIntrospectionError),
    /// An error that could occur when trying to get the value of a variable
    ValueGet(VarIntrospectionError),
    /// An error occurring when sending info to the internal mpsc channel.
    /// This is automatically converted to a string in the implementation of Into for simplicity
    Send(String),
}
impl std::error::Error for VarError {}
impl fmt::Display for VarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Typeset(e) => write!(f, "Error setting variable type: {}", e),
            Self::ValueSet(e) => write!(f, "Error setting variable value: {}", e),
            Self::ValueUnset(e) => write!(f, "Error unsetting variable: {}", e),
            Self::ValueGet(e) => write!(f, "Error getting variable value: {}", e),
            Self::Send(e) => write!(f, "Error sending to internal mpsc channel: {}", e),
        }
    }
}
impl<T> From<std::sync::mpsc::SendError<T>> for VarError
where
    T: fmt::Display,
{
    fn from(e: std::sync::mpsc::SendError<T>) -> Self {
        Self::Send(e.to_string())
    }
}
impl From<VarTypesetError> for VarError {
    fn from(e: VarTypesetError) -> Self {
        Self::Typeset(e)
    }
}
