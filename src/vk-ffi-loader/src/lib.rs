extern crate vk_ffi;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadError(pub &'static str);

impl From<&'static str> for LoadError {
    fn from(val: &'static str) -> Self { LoadError(val) }
}

impl From<LoadError> for &'static str {
    fn from(val: LoadError) -> Self { val.0 }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "failed to load function '{}'", self.0)
    }
}

impl std::error::Error for LoadError {}

// Easier to implement this manually than to rig the generator to do it.
pub mod entry {
    macro_rules! impl_entry {
        ($($member:ident: $type:ty = $symbol:expr,)*) => {
            use crate::LoadError;
            #[derive(Clone, Copy)]
            pub struct Entry { $(pub $member: $type,)* }
            impl Entry {
                pub unsafe fn load(
                    get_instance_proc_addr: ::vk_ffi::FnGetInstanceProcAddr,
                ) -> ::std::result::Result<Self, crate::LoadError> {
                    Ok(Entry {$(
                        $member: ::std::mem::transmute({
                            get_instance_proc_addr(
                                ::vk_ffi::null(),
                                $symbol.as_ptr() as *const _,
                            ).ok_or_else(|| {
                                LoadError(::std::str::from_utf8_unchecked(
                                    &$symbol[..$symbol.len() - 1]
                                ))
                            })?
                        }),
                    )*})
                }
            }
        }
    }
    pub mod v1_0 {
        impl_entry! {
            enumerate_instance_extension_properties:
                ::vk_ffi::FnEnumerateInstanceExtensionProperties
                = b"vkEnumerateInstanceExtensionProperties\0",
            enumerate_instance_layer_properties:
                ::vk_ffi::FnEnumerateInstanceLayerProperties
                = b"vkEnumerateInstanceLayerProperties\0",
            create_instance:
                ::vk_ffi::FnCreateInstance
                = b"vkCreateInstance\0",
        }
    }
    pub mod v1_1 {
        impl_entry! {
            enumerate_instance_version: ::vk_ffi::FnEnumerateInstanceVersion
                = b"vkEnumerateInstanceVersion\0",
            enumerate_instance_extension_properties:
                ::vk_ffi::FnEnumerateInstanceExtensionProperties
                = b"vkEnumerateInstanceExtensionProperties\0",
            enumerate_instance_layer_properties:
                ::vk_ffi::FnEnumerateInstanceLayerProperties
                = b"vkEnumerateInstanceLayerProperties\0",
            create_instance:
                ::vk_ffi::FnCreateInstance
                = b"vkCreateInstance\0",
        }
    }
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/loader.rs"));
