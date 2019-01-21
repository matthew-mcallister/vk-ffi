extern crate vk_ffi;

use vk_ffi::*;

macro_rules! opt_to_ptr {
    ($opt:expr) => { $opt.map_or(std::ptr::null(), |ptr| ptr as *const ()) }
}

// Possibly easier to implement this manually than to rig the generator to.
macro_rules! impl_entry {
    ($(
        $member:ident {
            params: ($($param:ident: $param_ty:ty,)*),
            pfn_ty: $pfn_ty:ty,
            fn_ty: $fn_ty:ty,
            symbol: $symbol:expr,
        },
    )*) => {
        #[derive(Clone, Copy)]
        pub struct Entry { $(pub $member: $pfn_ty,)* }
        impl Entry {
            pub unsafe fn load(
                get_instance_proc_addr: ::vk_ffi::FnGetInstanceProcAddr,
            ) -> Self {
                Entry {$(
                    $member: ::std::mem::transmute({
                        get_instance_proc_addr(
                            ::vk_ffi::null(),
                            $symbol as *const _ as *const _ as *const _,
                        )
                    }),
                )*}
            }

            $(
                pub unsafe fn $member(&self, $($param: $param_ty,)*)
                    -> ::vk_ffi::Result
                {
                    ::std::mem::transmute::<_, $fn_ty>(self.$member)
                        ($($param,)*)
                }
            )*
        }
        impl std::fmt::Debug for Entry {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct("Entry")
                    $(.field(stringify!($member), &opt_to_ptr!(self.$member)))*
                    .finish()
            }
        }
    }
}

impl_entry! {
    enumerate_instance_extension_properties {
        params: (
            p_layer_name: *const ::std::os::raw::c_char,
            p_property_count: *mut u32,
            p_properties: *mut ::vk_ffi::ExtensionProperties,
        ),
        pfn_ty: PfnEnumerateInstanceExtensionProperties,
        fn_ty: FnEnumerateInstanceExtensionProperties,
        symbol: b"vkEnumerateInstanceExtensionProperties\0",
    },
    enumerate_instance_layer_properties {
        params: (
            p_property_count: *mut u32,
            p_properties: *mut ::vk_ffi::LayerProperties,
        ),
        pfn_ty: PfnEnumerateInstanceLayerProperties,
        fn_ty: FnEnumerateInstanceLayerProperties,
        symbol: b"vkEnumerateInstanceLayerProperties\0",
    },
    create_instance {
        params: (
            p_create_info: *const ::vk_ffi::InstanceCreateInfo,
            p_allocator: *const ::vk_ffi::AllocationCallbacks,
            p_instance: *mut ::vk_ffi::Instance,
        ),
        pfn_ty: PfnCreateInstance,
        fn_ty: FnCreateInstance,
        symbol: b"vkCreateInstance\0",
    },
    enumerate_instance_version {
        params: (
            p_api_version: *mut u32,
        ),
        pfn_ty: PfnEnumerateInstanceVersion,
        fn_ty: FnEnumerateInstanceVersion,
        symbol: b"vkEnumerateInstanceVersion\0",
    },
}

macro_rules! declare_api {
    ($name:ident {
        $owner:ident: $owner_ty:ident,
        $($member:ident: $type:ty,)*
    }) => {
        #[derive(Clone, Copy)]
        pub struct $name {
            pub $owner: $owner_ty,
            $(pub $member: $type,)*
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct(stringify!($name))
                    .field(stringify!($owner), &self.$owner)
                    $(.field(stringify!($member), &opt_to_ptr!(self.$member)))*
                    .finish()
            }
        }
    }
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/loader.rs"));
