#![feature(try_blocks)]

extern crate libloading as lib;
#[macro_use]
extern crate vk_ffi as vk;
extern crate vk_ffi_loader;

use std::ptr;

use vk_ffi_loader::v1_0 as vkl;

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

#[macro_export]
macro_rules! data_file {
    ($str:expr) => {
        concat!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/"), $str)
    }
}

#[macro_export]
macro_rules! output_file {
    ($str:expr) => {
        concat!(concat!(env!("CARGO_MANIFEST_DIR"), "/output/"), $str)
    }
}

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

pub struct VulkanSys {
    pub loader: Loader,
    pub entry: vkl::Entry,
    pub instance: vkl::CoreInstance,
    pub physical_device: vk::PhysicalDevice,
    pub device: vkl::CoreDevice,
    pub queue: vk::Queue,
}

impl Drop for VulkanSys {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(ptr::null());
            self.instance.destroy_instance(ptr::null());
        }
    }
}

impl VulkanSys {
    pub unsafe fn new() -> Self {
        let loader = Loader::load();
        let entry = vkl::Entry::load(loader.get_instance_proc_addr).unwrap();

        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: c_str!("vk-ffi demo"),
            application_version: vk_make_version!(0, 1, 0),
            p_engine_name: ptr::null(),
            engine_version: vk_make_version!(0, 1, 0),
            api_version: vk::API_VERSION_1_0,
        };
        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            p_application_info: &app_info as *const _,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
        };
        let mut vk_instance = vk::null();
        entry.create_instance
            (&create_info as *const _, ptr::null(), &mut vk_instance as *mut _)
            .check().unwrap();

        let instance = vkl::CoreInstance::load
            (vk_instance, loader.get_instance_proc_addr)
            .unwrap();

        let physical_devices =
            vk_enumerate2!(instance, enumerate_physical_devices).unwrap();
        let physical_device = physical_devices[0];

        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_family_index: 0,
            queue_count: 1,
            p_queue_priorities: &1.0f32 as *const _,
        };
        let features: vk::PhysicalDeviceFeatures = Default::default();
        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info as *const _,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
            p_enabled_features: &features as *const _,
        };
        let mut vk_device = vk::null();
        instance.create_device(
            physical_device,
            &create_info as *const _,
            ptr::null(),
            &mut vk_device as *mut _,
        ).check().unwrap();

        let device = vkl::CoreDevice::load
            (vk_device, loader.get_device_proc_addr)
            .unwrap();

        let mut queue = vk::null();
        device.get_device_queue(0, 0, &mut queue as *mut _);

        VulkanSys {
            loader,
            entry,
            instance,
            physical_device,
            device,
            queue,
        }
    }
}

#[cfg(target_endian = "little")]
pub unsafe fn from_bytes_le<T: Sized>(bytes: &[u8]) -> &[T] {
    assert_eq!(bytes.len() % std::mem::size_of::<T>(), 0);
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const T,
        bytes.len() / std::mem::size_of::<T>(),
    )
}

#[cfg(target_endian = "big")]
pub unsafe fn from_bytes_le<T: Sized>(bytes: &[u8]) -> &[T] {
    panic!("target platform is not little endian");
}
