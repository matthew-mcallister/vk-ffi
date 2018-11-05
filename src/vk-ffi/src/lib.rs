#![feature(extern_types)]

use std::ops::*;

#[macro_use]
mod macros;

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

pub trait VkHandle: Eq + Sized {
    fn null() -> Self;
    fn is_null(self) -> bool { self == Self::null() }
}

pub fn null<T: VkHandle>() -> T { <T as VkHandle>::null() }

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/bindings.rs"));

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

impl ::std::fmt::Display for Result {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", match *self {
            Result::SUCCESS => "success",
            Result::NOT_READY => "not ready",
            Result::TIMEOUT => "operation timed out",
            Result::EVENT_SET => "event signaled",
            Result::EVENT_RESET => "event unsignaled",
            Result::INCOMPLETE => "incomplete result",
            Result::ERROR_OUT_OF_HOST_MEMORY => "out of host memory",
            Result::ERROR_OUT_OF_DEVICE_MEMORY => "out of device memory",
            Result::ERROR_INITIALIZATION_FAILED => "initialization failed",
            Result::ERROR_DEVICE_LOST => "device lost",
            Result::ERROR_MEMORY_MAP_FAILED => "memory map failed",
            Result::ERROR_LAYER_NOT_PRESENT => "layer not present",
            Result::ERROR_EXTENSION_NOT_PRESENT => "extension not present",
            Result::ERROR_FEATURE_NOT_PRESENT => "feature not present",
            Result::ERROR_INCOMPATIBLE_DRIVER => "incompatible driver",
            Result::ERROR_TOO_MANY_OBJECTS => "too many objects",
            Result::ERROR_FORMAT_NOT_SUPPORTED => "format not supported",
            Result::ERROR_FRAGMENTED_POOL => "fragmented object pool",
            Result::ERROR_OUT_OF_POOL_MEMORY => "object pool out of memory",
            Result::ERROR_INVALID_EXTERNAL_HANDLE => "invalid external handle",
            Result::ERROR_SURFACE_LOST_KHR => "surface lost",
            Result::ERROR_NATIVE_WINDOW_IN_USE_KHR => "native window in use",
            Result::SUBOPTIMAL_KHR => "suboptimal swapchain",
            Result::ERROR_OUT_OF_DATE_KHR => "swapchain out of date",
            Result::ERROR_INCOMPATIBLE_DISPLAY_KHR => "incompatible display",
            Result::ERROR_VALIDATION_FAILED_EXT => "validation failed",
            Result::ERROR_INVALID_SHADER_NV => "invalid shader",
            Result::ERROR_INVALID_DRM_FORMAT_MODIFIER_PLANE_LAYOUT_EXT =>
                "invalid DRM format modifier plane layout",
            Result::ERROR_FRAGMENTATION_EXT => "memory fragmentation",
            Result::ERROR_NOT_PERMITTED_EXT => "operation not permitted",
            _ => "unrecognized status code",
        })
    }
}

impl ::std::error::Error for Result {}
