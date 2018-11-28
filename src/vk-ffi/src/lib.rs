#![feature(extern_types)]

use std::ops::*;

#[macro_use]
mod macros;
mod imp;

pub use self::macros::*;

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

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/bindings.rs"));

// These are given incorrect types by bindgen
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

// bindgen skips these because they are defined by macro calls
pub const API_VERSION_1_0: u32 = crate::make_version!(1, 0, 0);
pub const API_VERSION_1_1: u32 = crate::make_version!(1, 1, 0);

pub trait VkHandle: Eq + Sized {
    #[inline]
    fn null() -> Self;
    #[inline]
    fn is_null(self) -> bool { self == Self::null() }
}

pub fn null<T: VkHandle>() -> T { <T as VkHandle>::null() }

impl Result {
    #[inline]
    pub fn check(self) -> ::std::result::Result<Self, Self> {
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

        impl ::std::convert::From<$name> for ($($type,)*) {
            #[inline]
            fn from($name { $($field,)* }: $name) -> Self {
                ($($field,)*)
            }
        }

        impl ::std::convert::From<($($type,)*)> for $name {
            #[inline]
            fn from(($($field,)*): ($($type,)*)) -> Self {
                $name { $($field,)* }
            }
        }

        impl ::std::cmp::PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                true
                    $(&& self.$field == other.$field)*
            }
        }

        impl ::std::cmp::Eq for $name {}
    }
}

impl_tuple_like!(Offset2D { x: i32, y: i32, });
impl_tuple_like!(Offset3D { x: i32, y: i32, z: i32, });
impl_tuple_like!(Extent2D { width: u32, height: u32, });
impl_tuple_like!(Extent3D { width: u32, height: u32, depth: u32, });
impl_tuple_like!(Rect2D { offset: Offset2D, extent: Extent2D, });
