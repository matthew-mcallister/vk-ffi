#![feature(try_blocks)]

use std::ptr;

use examples::*;

unsafe fn unsafe_main() {
    let loader = Loader::load();
    let entry = vkl::Entry::load(loader.get_instance_proc_addr);

    let enabled_layers = [VALIDATION_LAYER.as_ptr() as _];

    let create_info = vk::InstanceCreateInfo {
        p_application_info: ptr::null(),
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

    let queue_create_infos = [vk::DeviceQueueCreateInfo {
        queue_family_index: 0,
        queue_count: 1,
        p_queue_priorities: &1.0f32,
        ..Default::default()
    }];
    let create_info = vk::DeviceCreateInfo {
        queue_create_info_count: queue_create_infos.len() as _,
        p_queue_create_infos: queue_create_infos.as_ptr(),
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

    let create_info = vk::BufferCreateInfo {
        size: 0x1_0000,
        usage: vk::BufferUsageFlags::INDEX_BUFFER_BIT,
        ..Default::default()
    };
    let mut buffer = vk::null();
    device.create_buffer(&create_info, ptr::null(), &mut buffer);

    // Don't free the buffer, causing a validation error
    device.destroy_device(ptr::null());
    instance.destroy_instance(ptr::null());
}

fn main() {
    unsafe { unsafe_main() }
}
