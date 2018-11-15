#![feature(extern_types)]

use std::ops::*;

#[macro_use]
mod macros;
mod imp;

macro_rules! impl_unary_op {
    ($OpName:ident, $opname:ident; $name:ident) => {
        impl $OpName for $name {
            type Output = Self;
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
            fn $opname(self, other: Self) -> Self {
                $name((self.0).$opname(other.0))
            }
        }
    }
}

macro_rules! impl_bin_op_assign {
    ($OpAssign:ident, $opassign:ident; $name:ident) => {
        impl $OpAssign for $name {
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
            pub fn empty() -> Self { $name(0) }
            pub fn is_empty(self) -> bool { self.0 == 0 }
            pub fn intersects(self, other: Self) -> bool
                { self.bitand(other).0 != 0 }
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
pub const API_VERSION_1_0: u32 = vk_make_version!(1, 0, 0);
pub const API_VERSION_1_1: u32 = vk_make_version!(1, 1, 0);

pub trait VkHandle: Eq + Sized {
    fn null() -> Self;
    fn is_null(self) -> bool { self == Self::null() }
}

pub fn null<T: VkHandle>() -> T { <T as VkHandle>::null() }

impl Result {
    pub fn check(self) -> ::std::result::Result<Self, Self> {
        vk_check!(self)
    }

    pub fn is_success(self) -> bool {
        self.0 >= 0
    }

    pub fn is_error(self) -> bool {
        self.0 < 0
    }
}
