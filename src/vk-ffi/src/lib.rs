#![feature(extern_types)]
#![feature(trait_alias)]
#![allow(non_upper_case_globals)]

use std::os::raw::*;

#[macro_use]
mod macros;
mod imp;
#[cfg(test)]
mod tests;

pub use crate::macros::*;

// Enums and bitmasks

macro_rules! impl_unary_op {
    ($OpName:ident, $opname:ident; $name:ident) => {
        impl $OpName for $name {
            type Output = Self;
            #[inline]
            fn $opname(self) -> Self {
                $name((self.0).$opname())
            }
        }
    }
}

macro_rules! impl_bin_op {
    ($OpName:ident, $opname:ident; $name:ident) => {
        impl $OpName for $name {
            type Output = Self;
            #[inline]
            fn $opname(self, other: Self) -> Self {
                $name((self.0).$opname(other.0))
            }
        }
    }
}

macro_rules! impl_bin_op_assign {
    ($OpAssign:ident, $opassign:ident; $name:ident) => {
        impl $OpAssign for $name {
            #[inline]
            fn $opassign(&mut self, other: Self) {
                (self.0).$opassign(other.0)
            }
        }
    }
}

macro_rules! bitmask_impls {
    ($name:ident) => {
        impl_unary_op!(Not, not; $name);
        impl_bin_op!(BitAnd, bitand; $name);
        impl_bin_op_assign!(BitAndAssign, bitand_assign; $name);
        impl_bin_op!(BitOr, bitor; $name);
        impl_bin_op_assign!(BitOrAssign, bitor_assign; $name);
        impl_bin_op!(BitXor, bitxor; $name);
        impl_bin_op_assign!(BitXorAssign, bitxor_assign; $name);
        impl $name {
            #[inline]
            pub fn empty() -> Self { $name(0) }
            #[inline]
            pub fn all() -> Self { $name(!0) }
            #[inline]
            pub fn is_empty(self) -> bool { self.0 == 0 }
            #[inline]
            pub fn intersects(self, other: Self) -> bool
                { self.bitand(other).0 != 0 }
            #[inline]
            pub fn contains(self, other: Self) -> bool
                { self.bitand(other).0 == other.0 }
        }
    }
}

macro_rules! impl_enum {
    (@inner $name:ident[$type:ty] {$($member:ident = $value:expr,)*}) => {
        #[repr(transparent)]
        #[derive(
            Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd,
        )]
        pub struct $name(pub $type);
        impl $name {
            $(pub const $member: $name = $name($value);)*
        }
        #[cfg(feature = "reflection")]
        impl std::str::FromStr for $name {
            type Err = crate::reflection::ParseEnumError;
            #[allow(unreachable_code)]
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Ok(match s {
                    $(stringify!($member) => $name::$member,)*
                    _ => return Err(Default::default()),
                })
            }
        }
        #[cfg(feature = "reflection")]
        impl crate::reflection::EnumInfo for $name {
            const MEMBERS: &'static [&'static str] =
                &[$(stringify!($member),)*];
            const VALUES: &'static [Self] = &[$($name::$member,)*];
        }
        impl From<$type> for $name {
            fn from(val: $type) -> Self { $name(val) }
        }
        impl From<$name> for $type {
            fn from(val: $name) -> Self { val.0 }
        }
    };
    (enum $name:ident {$($member:ident = $value:expr,)*}) => {
        impl_enum!(@inner $name[i32] {$($member = $value,)*});
    };
    (bitmask $name:ident {$($member:ident = $value:expr,)*}) => {
        impl_enum!(@inner $name[u32] {$($member = $value,)*});
        bitmask_impls!($name);
    };
}

macro_rules! impl_enums {
    ($($ty:ident $name:ident $body:tt;)*) => {
        mod enums {
            use std::ops::*;
            $(impl_enum!($ty $name $body);)*
        }
    }
}

// Aliases and extensions

macro_rules! impl_aliases {
    ($($name:ident = $target:path;)*) => {
        mod aliases {
            $(pub type $name = $target;)*
        }
    }
}

macro_rules! impl_extensions {
    ($($name:ident = $val:ident;)*) => {
        $(
            pub const $name: *const c_char =
                concat!(stringify!($val), "\0") as *const str as *const _;
        )*
    }
}

// Handles

macro_rules! impl_handle {
    ($name:ident { dispatchable: true }) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub *const std::ffi::c_void);
        impl crate::traits::HandleType for $name {
            #[inline]
            fn null() -> Self { $name(0 as *const _) }
            #[inline]
            fn is_null(self) -> bool { self.0 as usize == 0 }
        }
        impl std::default::Default for $name {
            #[inline]
            fn default() -> Self { crate::null() }
        }
    };
    ($name:ident { dispatchable: false }) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub u64);
        impl crate::traits::HandleType for $name {
            #[inline]
            fn null() -> Self { $name(0) }
            #[inline]
            fn is_null(self) -> bool { self.0 == 0 }
        }
        impl std::default::Default for $name {
            #[inline]
            fn default() -> Self { crate::null() }
        }
    }
}

macro_rules! impl_handles {
    ($($name:ident { dispatchable: $value:ident };)*) => {
        mod handles { $(impl_handle!($name { dispatchable: $value });)* }
    }
}

// Structs and unions

macro_rules! impl_aggregate {
    (@inner $kw:tt $name:ident { $($member:ident: $type:ty,)* }) => {
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub $kw $name { $(pub $member: $type,)* }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, concat!(stringify!($name), " {{ ... }}"))
            }
        }
        #[cfg(feature = "reflection")]
        impl crate::reflection::AggregateInfo for $name {
            const FIELDS: &'static [&'static str] =
                &[$(stringify!($member),)*];
        }
    };
    (
        struct $name:ident {
            $($member:ident: $type:ty $(= $default:expr)*,)*
        }
    ) => {
        impl_aggregate!(@inner struct $name { $($member: $type,)* });
        impl Default for $name {
            #[inline]
            fn default() -> Self {
                $name {
                    $($($member: $default,)*)*
                    ..unsafe { std::mem::zeroed() }
                }
            }
        }
    };
    (union $name:ident { $($member:ident: $type:ty,)* }) => {
        impl_aggregate!(@inner union $name { $($member: $type,)* });
        impl Default for $name {
            #[inline]
            fn default() -> Self { unsafe { std::mem::zeroed() } }
        }
    };
}

macro_rules! impl_aggregates {
    ($($ty:ident $name:ident $body:tt;)*) => {
        mod aggregates {
            use std::os::raw::*;
            $(impl_aggregate!($ty $name $body);)*
        }
    }
}

// Function pointers and commands

macro_rules! impl_func_pointers {
    ($($name:ident ($($arg:ident: $type:ty,)*) $(-> $ret:ty)*;)*) => {
        mod fn_ptrs {
            use std::ffi::c_void;
            use std::os::raw::*;
            $(
                pub type $name =
                    Option<unsafe extern "C" fn($($arg: $type,)*) $(-> $ret)*>;
            )*
        }
    };
}

macro_rules! impl_commands {
    ($($name:ident ($($arg:ident: $type:ty,)*) $(-> $ret:ty)*;)*) => {
        mod cmds {
            use std::ffi::c_void;
            use std::os::raw::*;
            $(
                pub type $name =
                    unsafe extern "C" fn($($arg: $type,)*) $(-> $ret)*;
            )*
        }
    }
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/bindings.rs"));

// This convoluted module layout exists solely to work around name
// clashes between data types and function pointers.

// This module represents the "VK*" types
mod data {
    pub use crate::aliases::*;
    pub use crate::handles::*;
    pub use crate::enums::*;
    pub use crate::aggregates::*;
}

// This module represents the "PFN_vk*" types
pub mod pfn {
    pub use crate::fn_ptrs::*;
    pub use crate::cmds::*;
}

pub use data::*;

// TODO: Could stand to make Bool32 an enum
pub const FALSE: Bool32 = 0;
pub const TRUE: Bool32 = 1;

pub const LOD_CLAMP_NONE: f32 = 1000.0;
pub const REMAINING_MIP_LEVELS: u32 = !0u32;
pub const REMAINING_ARRAY_LAYERS: u32 = !0u32;
pub const WHOLE_SIZE: u64 = !0u64;
pub const ATTACHMENT_UNUSED: u32 = !0u32;
pub const QUEUE_FAMILY_IGNORED: u32 = !0u32;
pub const QUEUE_FAMILY_EXTERNAL: u32 = !0u32 - 1;
pub const QUEUE_FAMILY_EXTERNAL_KHR: u32 = QUEUE_FAMILY_EXTERNAL;
pub const QUEUE_FAMILY_FOREIGN_EXT: u32 = !0u32 - 2;
pub const SUBPASS_EXTERNAL: u32 = !0u32;

pub const API_VERSION_1_0: u32 = crate::make_version!(1, 0, 0);
pub const API_VERSION_1_1: u32 = crate::make_version!(1, 1, 0);

#[cfg(feature = "reflection")]
pub mod reflection {
    pub trait EnumInfo: 'static + std::str::FromStr {
        const MEMBERS: &'static [&'static str];
        const VALUES: &'static [Self];
    }

    pub trait AggregateInfo: 'static {
        const FIELDS: &'static [&'static str];
    }

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct ParseEnumError { _priv: () }

    impl std::fmt::Display for ParseEnumError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "unknown enum member")
        }
    }

    impl std::error::Error for ParseEnumError {}
}

pub mod traits {
    use std::fmt::Debug;
    use std::ops::*;

    pub trait HandleType: Eq + Sized {
        #[inline]
        fn null() -> Self;

        #[inline]
        fn is_null(self) -> bool { self == Self::null() }
    }

    pub trait Enum = Sized + Copy + Default + Debug + Eq + From<i32> +
        Into<i32>;

    pub trait Bitmask = Sized + Copy + Default + Debug + Eq + From<u32> +
        Into<u32> + Not + BitAnd + BitAndAssign + BitOr + BitOrAssign +
        BitXor + BitXorAssign;
}

/// Returns a null-valued handle.
pub fn null<T: crate::traits::HandleType>() -> T {
    <T as crate::traits::HandleType>::null()
}

impl Result {
    #[inline]
    pub fn check(self) -> std::result::Result<Self, Self> {
        crate::check!(self)
    }

    #[inline]
    pub fn is_success(self) -> bool {
        self.0 >= 0
    }
    #[inline]
    pub fn is_error(self) -> bool {
        self.0 < 0
    }
}

macro_rules! impl_tuple_like {
    ($name:ident { $($field:ident: $type:ty,)* }) => {
        impl $name {
            #[inline]
            pub fn new($($field: $type,)*) -> Self {
                $name { $($field,)* }
            }
        }

        impl From<$name> for ($($type,)*) {
            #[inline]
            fn from($name { $($field,)* }: $name) -> Self {
                ($($field,)*)
            }
        }

        impl From<($($type,)*)> for $name {
            #[inline]
            fn from(($($field,)*): ($($type,)*)) -> Self {
                $name { $($field,)* }
            }
        }

        impl PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                true
                    $(&& self.$field == other.$field)*
            }
        }

        impl Eq for $name {}
    }
}

impl_tuple_like!(Offset2D { x: i32, y: i32, });
impl_tuple_like!(Offset3D { x: i32, y: i32, z: i32, });
impl_tuple_like!(Extent2D { width: u32, height: u32, });
impl_tuple_like!(Extent3D { width: u32, height: u32, depth: u32, });
impl_tuple_like!(Rect2D { offset: Offset2D, extent: Extent2D, });
