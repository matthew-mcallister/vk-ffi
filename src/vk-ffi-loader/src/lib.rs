#![allow(unused_parens)]

use std::fmt::Debug;
use std::ffi::c_void;
use std::os::raw::*;

use vk_ffi::*;

macro_rules! vk_symbol {
    ($cmd:ident) => {
        concat!(concat!("vk", stringify!($cmd)), "\0")
            as *const str as *const c_char
    }
}

macro_rules! opt_to_ptr {
    ($opt:expr) => { $opt.map_or(std::ptr::null(), |p| p as *const c_void) }
}

// Easier to implement this manually than to rig the generator to.
macro_rules! impl_entry {
    (
        $(
            {
                name: $member:ident,
                method_name: $method:ident,
                ptr: $pfn:ident,
                signature: ($($arg:ident: $type:ty,)*) $(-> $ret:ty)*,
            },
        )*
    ) => {
        #[derive(Clone, Copy)]
        pub struct Entry { $(pub $member: Option<pfn::$pfn>,)* }
        impl Entry {
            pub unsafe fn load(get_proc_addr: pfn::GetInstanceProcAddr) -> Self
            {
                Entry {
                    $(
                        $member: {
                            let symbol = vk_symbol!($pfn);
                            std::mem::transmute(get_proc_addr(null(), symbol))
                        },
                    )*
                }
            }
        }
        impl Debug for Entry {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct(stringify!(Entry))
                    $(.field(stringify!($member), &opt_to_ptr!(self.$member)))*
                    .finish()
            }
        }
        impl Entry {
            $(
                pub unsafe fn $method(&self, $($arg: $type,)*) $(-> $ret)* {
                    std::mem::transmute::<_, pfn::$pfn>(self.$member)
                        ($($arg,)*)
                }
            )*
        }
    }
}

impl_entry! {
    {
        name: pfn_enumerate_instance_extension_properties,
        method_name: enumerate_instance_extension_properties,
        ptr: EnumerateInstanceExtensionProperties,
        signature: (
            p_layer_name: *const c_char,
            p_property_count: *mut u32,
            p_properties: *mut ExtensionProperties,
        ) -> Result,
    },
    {
        name: pfn_enumerate_instance_layer_properties,
        method_name: enumerate_instance_layer_properties,
        ptr: EnumerateInstanceLayerProperties,
        signature: (
            p_property_count: *mut u32,
            p_properties: *mut LayerProperties,
        ) -> Result,
    },
    {
        name: pfn_create_instance,
        method_name: create_instance,
        ptr: CreateInstance,
        signature: (
            p_create_info: *const InstanceCreateInfo,
            p_allocator: *const AllocationCallbacks,
            p_instance: *mut Instance,
        ) -> Result,
    },
    {
        name: pfn_enumerate_instance_version,
        method_name: enumerate_instance_version,
        ptr: EnumerateInstanceVersion,
        signature: (
            p_api_version: *mut u32,
        ) -> Result,
    },
}

macro_rules! call_cmd {
    (
        fn: $fn:expr,
        args: [$($arg:expr,)*],
        handle: $handle:expr,
        takes_handle: true,
    ) => {
        $fn($handle, $($arg,)*)
    };
    (
        fn: $fn:expr,
        args: [$($arg:expr,)*],
        handle: $handle:expr,
        takes_handle: false,
    ) => {
        $fn($($arg,)*)
    };
}

macro_rules! impl_table {
    (
        name: $name:ident,
        get_proc_addr: $get_proc_addr:ident,
        handle: {
            name: $handle:ident,
            type: $handle_type:ty,
        },
        members: [
            $(
                {
                    name: $member:ident,
                    method_name: $method:ident,
                    ptr: $pfn:ident,
                    signature: ($($arg:ident: $type:ty,)*) $(-> $ret:ty)*,
                    takes_handle: $takes_handle:tt,
                },
            )*
        ],
    ) => {
        #[derive(Clone, Copy)]
        pub struct $name {
            pub $handle: $handle_type,
            $(pub $member: Option<pfn::$pfn>,)*
        }
        impl $name {
            pub unsafe fn load(
                $handle: $handle_type,
                get_proc_addr: pfn::$get_proc_addr,
            ) -> Self {
                $name {
                    $handle,
                    $(
                        $member: {
                            let symbol = vk_symbol!($pfn);
                            std::mem::transmute(get_proc_addr($handle, symbol))
                        },
                    )*
                }
            }
        }
        impl Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field(stringify!($handle), &self.$handle.0)
                    $(.field(stringify!($member), &opt_to_ptr!(self.$member)))*
                    .finish()
            }
        }
        impl $name {
            $(
                pub unsafe fn $method(&self, $($arg: $type,)*) $(-> $ret)* {
                    call_cmd! {
                        fn: std::mem::transmute::<_, pfn::$pfn>(self.$member),
                        args: [$($arg,)*],
                        handle: self.$handle,
                        takes_handle: $takes_handle,
                    }
                }
            )*
        }
    }
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/loader.rs"));
