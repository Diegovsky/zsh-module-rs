use std::ffi::{c_char, CStr};

use zsh_sys as zsys;

use crate::{types::cstring::ManagedCStr, CStrArray, ToCString};

// Taken from Src/zsh.h
// TODO: generate this automatically from zsh
bitflags::bitflags! {
    pub struct ParamFlags: i32 {
        const PM_SCALAR =	0	;
        const PM_ARRAY =	(1<<0)	;
        const PM_INTEGER =	(1<<1)	;
        const PM_EFLOAT =	(1<<2)	;
        const PM_FFLOAT =	(1<<3)	;
        const PM_HASHED =	(1<<4)	;

        const PM_LEFT =		(1<<5)	;
        const PM_RIGHT_B =	(1<<6)	;
        const PM_RIGHT_Z =	(1<<7)	;
        const PM_LOWER =	(1<<8)	;
        const PM_UPPER =	(1<<9)	;
        const PM_UNDEFINED =	(1<<9)	;
        const PM_READONLY =	(1<<10)	;
        const PM_TAGGED =	(1<<11)	;
        const PM_EXPORTED =	(1<<12)	;
        const PM_ABSPATH_USED = (1<<12) ;
        const PM_UNIQUE =	(1<<13)	;
        const PM_UNALIASED =	(1<<13)	;
        const PM_HIDE =		(1<<14)	;
        const PM_CUR_FPATH =    (1<<14) ;
        const PM_HIDEVAL =	(1<<15)	;
        const PM_WARNNESTED =   (1<<15) ;
        const PM_TIED = 	(1<<16)	;
        const PM_TAGGED_LOCAL = (1<<16) ;
        const PM_DONTIMPORT_SUID = (1<<17) ;
        const PM_LOADDIR =      (1<<17) ;
        const PM_SINGLE =       (1<<18) ;
        const PM_ANONYMOUS =    (1<<18) ;
        const PM_LOCAL =	(1<<19) ;
        const PM_KSHSTORED =	(1<<19) ;
        const PM_SPECIAL =	(1<<20) ;
        const PM_ZSHSTORED =	(1<<20) ;
        const PM_RO_BY_DESIGN = (1<<21) ;
        const PM_READONLY_SPECIAL = (Self::PM_SPECIAL.bits|Self::PM_READONLY.bits|Self::PM_RO_BY_DESIGN.bits);
        const PM_DONTIMPORT =	(1<<22)	;
        const PM_DECLARED =	(1<<22) ;
        const PM_RESTRICTED =	(1<<23) ;
        const PM_UNSET =	(1<<24)	;
        const PM_DEFAULTED =	(Self::PM_DECLARED.bits|Self::PM_UNSET.bits);
        const PM_REMOVABLE =	(1<<25)	;
        const PM_AUTOLOAD =	(1<<26) ;
        const PM_NORESTORE =	(1<<27)	;
        const PM_AUTOALL =	(1<<27) ;
        const PM_HASHELEM =     (1<<28) ;
        const PM_NAMEDDIR =     (1<<29) ;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    Scalar,
    Integer,
    EFloat,
    FFloat,
    Array,
    Hashed,
}

impl ParamFlags {
    fn only_type(&self) -> ParamType {
        let only_type = *self
            & (Self::PM_SCALAR
                | Self::PM_INTEGER
                | Self::PM_EFLOAT
                | Self::PM_FFLOAT
                | Self::PM_ARRAY
                | Self::PM_HASHED);
        match only_type {
            Self::PM_SCALAR => ParamType::Scalar,
            Self::PM_INTEGER => ParamType::Integer,
            Self::PM_EFLOAT => ParamType::EFloat,
            Self::PM_FFLOAT => ParamType::FFloat,
            Self::PM_ARRAY => ParamType::Array,
            Self::PM_HASHED => ParamType::Hashed,
            _ => unreachable!("Uknown type"),
        }
    }
}

impl std::fmt::Debug for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Param")
            .field("type", &self.flags())
            .finish()
    }
}

macro_rules! gsu_wrapper {
    ($(struct $ident:ident ($raw:ty) -> $T:ty);* $(;)?) => {
        $(
        struct $ident<'a>(&'a $raw, zsys::Param);
        impl<'a> $ident<'a> {
            #[inline]
            unsafe fn new(raw: *const $raw, param: &'a mut Param) -> Self {
                Self(&*raw, param.as_mut_ptr())
            }
            #[inline]
            unsafe fn get(&self) -> $T {
                (self.0.getfn.expect("Missing getfn"))(self.1)
            }
            /* #[inline]
            unsafe fn set(&self, val: $T) {
                (self.0.setfn.expect("Missing setfn"))(self.1, val)
            }
            #[inline]
            unsafe fn unset(&self, flags: c_int) {
                (self.0.unsetfn.expect("Missing unsetfn"))(self.1, flags)
            } */

        })*
    };
}

gsu_wrapper! {
    struct GsuScalar(zsys::gsu_scalar) -> *mut c_char;
    struct GsuInteger(zsys::gsu_integer) -> zsys::zlong;
    struct GsuFloat(zsys::gsu_float) -> f64;
    struct GsuArray(zsys::gsu_array) -> *mut *mut c_char;
}

macro_rules! fn_get_gsu {
    ($name:ident, $field:ident, $gsu:ty) => {
        #[inline]
        unsafe fn $name<'a>(&'a mut self) -> $gsu {
            <$gsu>::new(self.0.gsu.$field, self)
        }
    };
}

/// A Zsh `Param`. This corresponds to a value inside Zsh.
#[repr(transparent)]
pub struct Param(zsys::param);

impl Param {
    /// A wrapper function that returns a [`Param`] from the current zsh internal `paramtab`.
    ///
    /// This will return [`None`] if the param does not exist.
    #[inline]
    pub fn new(name: impl ToCString) -> Option<Self> {
        get(name)
    }
    fn as_mut_ptr(&mut self) -> zsys::Param {
        &mut self.0
    }
    #[inline]
    pub fn flags(&self) -> ParamFlags {
        ParamFlags::from_bits(self.0.node.flags).unwrap()
    }

    fn_get_gsu!(scalar_gsu, s, GsuScalar);
    fn_get_gsu!(int_gsu, i, GsuInteger);
    fn_get_gsu!(float_gsu, f, GsuFloat);
    fn_get_gsu!(array_gsu, a, GsuArray);

    #[inline]
    pub fn type_of(&self) -> ParamType {
        self.flags().only_type()
    }

    #[inline]
    pub fn get_value(&mut self) -> ParamValue {
        match self.type_of() {
            ParamType::Scalar => {
                ParamValue::Scalar(unsafe { CStr::from_ptr(self.scalar_gsu().get()) })
            }
            ParamType::EFloat | ParamType::FFloat => {
                ParamValue::Float(unsafe { self.float_gsu().get() })
            }
            ParamType::Integer => ParamValue::Integer(unsafe { self.int_gsu().get() }),
            ParamType::Array => {
                ParamValue::Array(unsafe { CStrArray::from_raw(self.array_gsu().get().cast()) })
            }
            ParamType::Hashed => ParamValue::HashTable,
        }
    }
}

/// The possible types a Zsh `Param` can be.
#[derive(Debug)]
pub enum ParamValue<'a> {
    Scalar(&'a CStr),
    Integer(i64),
    Float(f64),
    Array(CStrArray),
    HashTable,
}

/// Returns a [`Param`] from the current `paramtab`.
pub fn get(name: impl ToCString) -> Option<Param> {
    let name = name.into_cstr().into_owned();
    let og_name = name.clone();
    let mut name = ManagedCStr::new(name);
    let mut value: zsys::value = unsafe { std::mem::zeroed() };
    if unsafe { zsys::getvalue(&mut value, &mut name.ptr(), 1) }.is_null() {
        None
    } else {
        unsafe {
            assert_eq!(name.c_str(), &*og_name);
            Some(Param(*value.pm))
        }
    }
}
