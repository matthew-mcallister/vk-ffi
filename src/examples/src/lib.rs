extern crate libloading as lib;
#[macro_use]
extern crate vk_ffi as vk;
extern crate vk_ffi_loader;

#[derive(Debug)]
pub struct Loader {
    pub lib: lib::Library,
    pub get_instance_proc_addr: vk::FnGetInstanceProcAddr,
    pub get_device_proc_addr: vk::FnGetDeviceProcAddr,
}

#[cfg(unix)]
const VULKAN_LOADER_PATH: &'static str = "libvulkan.so";

impl Loader {
    pub unsafe fn load() -> Self {
        let lib = lib::Library::new(VULKAN_LOADER_PATH).unwrap();
        let get_instance_proc_addr =
                *lib.get(b"vkGetInstanceProcAddr\0").unwrap();
        let get_device_proc_addr =
                *lib.get(b"vkGetDeviceProcAddr\0").unwrap();
        Loader { lib, get_instance_proc_addr, get_device_proc_addr }
    }
}

pub const SHADER_DIR: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/data/shaders");

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version { major, minor, patch }
    }

    pub fn from_packed(val: u32) -> Self {
        Version {
            major: vk_version_major!(val),
            minor: vk_version_minor!(val),
            patch: vk_version_patch!(val),
        }
    }

    pub fn to_packed(self) -> u32 {
        vk_make_version!(self.major, self.minor, self.patch)
    }
}

impl From<u32> for Version {
    fn from(val: u32) -> Self { Version::from_packed(val) }
}

impl From<Version> for u32 {
    fn from(val: Version) -> Self { val.to_packed() }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[macro_export]
macro_rules! c_str {
    ($str:expr) => { concat!($str, "\0").as_ptr() as *const _ as *const _ }
}
