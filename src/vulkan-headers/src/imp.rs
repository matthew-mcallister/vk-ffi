//! This module implements some traits that couldn't be derived.

use crate::*;

impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
            _ => "unidentified status code",
        })
    }
}

impl std::error::Error for Result {}
