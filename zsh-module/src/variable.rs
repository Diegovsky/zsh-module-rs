use crate::Zerror;
use std::{
    collections::HashMap,
    fmt, iter,
    sync::{
        mpsc::{SendError, Sender},
        Arc,
    },
};

/// The type we're using for the name of the variable. May change in the future if need be.
pub type VariableKey = String;

/// The type we're sending to the internal mpsc channel
pub type MpscVarType = (String, VarType);

/// WIP definition of a variable
///
/// TODO: Integrate with zsh
///
/// TODO: This is an owned value right now. Its existence should be tied directly to the referenced variable in zsh.
#[derive(Debug)]
pub struct Variable {
    name: String,
    // values may or may not be initialized. That's like running `typeset -a array` with no values.
    value: Option<VarType>,
    pub read_only: bool,
    pub scope: Visibility,
    // pub mpsc_channel: Arc<Sender<MpscVarType>>,
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
    /// Update the value of this variable. This calls internal zsh functions
    ///
    /// TODO: Implement
    pub fn set(&mut self, value: VarType) -> Result<(), Zerror> {
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
    // pub fn get(&self)
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

/// A builder for variables, similar to the zsh builtin `typeset`
///
/// Variable is meant to be a reference to a shell variable. This is meant to be owned and whatnot.
#[derive(Debug)]
pub struct VariableBuilder {
    pub name: String,
    pub value: Option<VarType>,
    pub read_only: bool,
    pub scope: Visibility,
}
impl VariableBuilder {
    pub fn new<S>(name: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            name: name.as_ref().to_string(),
            value: None,
            read_only: false,
            scope: Visibility::Global,
        }
    }
    pub fn readonly(&mut self) -> &mut Self {
        self.read_only = true;
        self
    }
    pub fn scope(&mut self, scope: Visibility) -> &mut Self {
        self.scope = Visibility::Global;
        self
    }
    pub fn value(&mut self, value: VarType) -> &mut Self {
        self.value = Some(value);
        self
    }
    pub fn build(self) -> Variable {
        Variable {
            name: self.name,
            value: self.value,
            read_only: self.read_only,
            scope: self.scope,
        }
    }
}

/// A variable primitive. All variables in zsh are typed.
#[derive(Debug)]
pub enum Primitive {
    Scalar(String),
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
        Self::Scalar(String::new())
    }
}

/// The type of a variable, used internally
#[derive(Debug)]
pub enum VarType {
    Primitive(Primitive),
    Array(Vec<Primitive>),
    Association(HashMap<String, Primitive>),
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

/// The scope of a variable
#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    /// typeset -x
    Export,
    /// typeset -g
    Global,
    /// typest alone in a function, or local
    Local,
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

/// Errors that could occur when trying to change a variable type at runtime
#[derive(Debug, Clone, Copy)]
pub enum VarTypesetError {
    ReadOnly,
    Disallowed,
}
impl fmt::Display for VarTypesetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VarTypesetError::ReadOnly => write!(f, "Variable is read-only"),
            VarTypesetError::Disallowed => write!(f, "Variable is disallowed"),
        }
    }
}

/// Errors that could occur when interacting with the internal zsh variable
///
/// TODO: Discover more
#[derive(Debug, Clone, Copy)]
pub enum VarIntrospectionError {
    /// Variable is invalid in some weird way that wouldn't happen while using the shell language
    InvalidVariable,
    /// When the zsh internal paramtab == realparamtab check fails
    MisalignedParamTab,
    /// The variable doesn't exist
    Nonexistent,
}
impl fmt::Display for VarIntrospectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VarIntrospectionError::InvalidVariable => write!(f, "Invalid variable"),
            VarIntrospectionError::MisalignedParamTab => write!(f, "Misaligned paramtab"),
            VarIntrospectionError::Nonexistent => write!(f, "Variable doesn't exist"),
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
            VarError::Typeset(e) => write!(f, "Error setting variable type: {}", e),
            VarError::ValueSet(e) => write!(f, "Error setting variable value: {}", e),
            VarError::ValueUnset(e) => write!(f, "Error unsetting variable: {}", e),
            VarError::ValueGet(e) => write!(f, "Error getting variable value: {}", e),
            VarError::Send(e) => write!(f, "Error sending to internal mpsc channel: {}", e),
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
