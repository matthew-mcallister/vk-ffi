#![feature(try_blocks)]

use std::ffi::CStr;
use std::ptr;

#[macro_export]
macro_rules! c_str {
    ($str:expr) => {
        concat!($str, "\0")
            as *const _ as *const _ as *const std::os::raw::c_char
    }
}

#[cfg(unix)]
pub const VULKAN_LOADER_PATH: &'static str = "libvulkan.so";

// TODO: fall back on LUNARG if KHRONOS unavailable
pub const VALIDATION_LAYER: &'static [u8] = b"VK_LAYER_KHRONOS_validation\0";

#[derive(Debug)]
pub struct Loader {
    pub lib: lib::Library,
    pub get_instance_proc_addr: vk::pfn::GetInstanceProcAddr,
    pub get_device_proc_addr: vk::pfn::GetDeviceProcAddr,
}

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
            major: vk::version_major!(val),
            minor: vk::version_minor!(val),
            patch: vk::version_patch!(val),
        }
    }

    pub fn to_packed(self) -> u32 {
        vk::make_version!(self.major, self.minor, self.patch)
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

#[derive(Debug)]
pub struct VulkanSys {
    pub loader: Loader,
    pub entry: vkl::Entry,
    pub instance: vkl::InstanceTable,
    pub physical_device: vk::PhysicalDevice,
    pub device: vkl::DeviceTable,
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
        let entry = vkl::Entry::load(loader.get_instance_proc_addr);

        // Enable validation if available
        let layers =
            vk::enumerate2!(entry, enumerate_instance_layer_properties)
            .unwrap();
        let enable_validation = layers.iter().any(|layer| {
            CStr::from_ptr(&layer.layer_name as *const _ as *const _)
                == CStr::from_bytes_with_nul_unchecked(VALIDATION_LAYER)
        });

        let validation_layers = [VALIDATION_LAYER.as_ptr() as _];
        let enabled_layers =
            if enable_validation { &validation_layers[..] }
            else { &[][..] };

        // This is the minimum required to run validation layers. To log
        // validation messages, either use the debug_utils extension or
        // configure the validation layer settings file. See
        //   https://vulkan.lunarg.com/doc/sdk/latest/linux/layer_configuration.html

        // Create instance
        let app_info = vk::ApplicationInfo {
            p_application_name: c_str!("vk-ffi demo"),
            application_version: vk::make_version!(0, 1, 0),
            p_engine_name: ptr::null(),
            engine_version: vk::make_version!(0, 1, 0),
            api_version: vk::API_VERSION_1_0,
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_layer_count: enabled_layers.len() as _,
            pp_enabled_layer_names: enabled_layers.as_ptr(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
            ..Default::default()
        };
        let mut vk_instance = vk::null();
        entry.create_instance(&create_info, ptr::null(), &mut vk_instance)
            .check().unwrap();

        let instance = vkl::InstanceTable::load
            (vk_instance, loader.get_instance_proc_addr);

        // Create device
        let physical_devices =
            vk::enumerate2!(instance, enumerate_physical_devices).unwrap();
        let physical_device = physical_devices[0];

        let queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index: 0,
            queue_count: 1,
            p_queue_priorities: &1.0f32,
            ..Default::default()
        };
        let features: vk::PhysicalDeviceFeatures = Default::default();
        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
            p_enabled_features: &features,
            ..Default::default()
        };
        let mut vk_device = vk::null();
        instance.create_device(
            physical_device,
            &create_info,
            ptr::null(),
            &mut vk_device,
        ).check().unwrap();

        let device =
            vkl::DeviceTable::load(vk_device, loader.get_device_proc_addr);

        let mut queue = vk::null();
        device.get_device_queue(0, 0, &mut queue);

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
