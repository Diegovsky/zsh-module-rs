use std::{
    ffi::{c_char, c_int, c_long, c_uint, CStr},
    ptr::NonNull,
};

use zsh_sys as zsys;

use crate::{from_cstr, hashtable::RawHashTable, StringArray};

#[repr(C)]
/// A Zsh `Param`. This corresponds to a value inside Zsh. See [`paramtab()`] for more info.
pub struct Param {
    raw: zsys::param,
}

// Taken from Src/zsh.h
// TODO: generate this automatically from zsh
bitflags::bitflags! {
    struct ParamFlags: i32 {
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
enum ParamType {
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
        let node = &self.raw.node;
        let raw = &self.raw;
        let old = NonNull::new(Param::from_raw(raw.old)).map(|ptr| unsafe { ptr.as_ref() });
        f.debug_struct("Param")
            .field("name", &unsafe { from_cstr(node.nam) })
            .field("flags", &ParamFlags::from_bits(node.flags))
            .field("base", &raw.base)
            .field("width", &raw.width)
            .field("env", &unsafe { from_cstr(raw.env) })
            .field("ename", &unsafe { from_cstr(raw.ename) })
            .field("old", &old)
            .finish()
    }
}

/// The possible types a Zsh `Param` can be.
#[derive(Debug)]
#[non_exhaustive]
pub enum ParamValue<'a> {
    String(&'a CStr),
    Integer(i64),
    Float(f64),
    Array(StringArray),
    // Hashed,
}
