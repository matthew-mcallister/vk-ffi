#![feature(try_blocks)]

extern crate examples;
extern crate vk_ffi as vk;
extern crate vk_ffi_loader as vkl;

use std::ffi;
use std::ptr;

use examples::*;

fn main() {
    unsafe { unsafe_main() }
}

macro_rules! print_bits {
    ($target:expr; $($bit:expr => $name:expr,)*) => {
        let mut first = true;
        for &(bit, name) in [$(($bit, $name),)*].iter() {
            if !$target.intersects(bit) { continue; }
            if first { print!("{}", name); first = false; }
            else { print!(" | {}", name); }
        }
        if first { print!("0"); }
    }
}

unsafe fn unsafe_main() {
    let loader = Loader::load();
    let entry = vkl::Entry::load(loader.get_instance_proc_addr);

    let layers = vk::enumerate2!(entry, enumerate_instance_layer_properties)
        .unwrap();
    println!("layers:");
    for layer in layers.into_iter() {
        println!("  - name: {:?}",
            ffi::CStr::from_ptr(&layer.layer_name as *const _));
        println!("    spec_version: {}",
            Version::from(layer.spec_version));
        println!("    impl_version: {}",
            Version::from(layer.implementation_version));
        println!("    desc: {:?}",
            ffi::CStr::from_ptr(&layer.description as *const _));
    }

    let exts = vk::enumerate2!(
        entry,
        enumerate_instance_extension_properties,
        ptr::null(),
    ).unwrap();
    println!("extensions:");
    for ext in exts.into_iter() {
        println!("  - name: {:?}",
            ffi::CStr::from_ptr(&ext.extension_name as *const _));
        println!("    spec_version: {}",
            Version::from(ext.spec_version));
    }

    let app_info = vk::ApplicationInfo {
        p_application_name: c_str!("Diagnostic example"),
        application_version: vk::make_version!(0, 1, 0),
        p_engine_name: ptr::null(),
        engine_version: vk::make_version!(0, 1, 0),
        api_version: vk::API_VERSION_1_0,
        ..Default::default()
    };
    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info as *const _,
        enabled_layer_count: 0,
        pp_enabled_layer_names: ptr::null(),
        enabled_extension_count: 0,
        pp_enabled_extension_names: ptr::null(),
        ..Default::default()
    };
    let mut instance = vk::null();
    entry.create_instance
        (&create_info as *const _, ptr::null(), &mut instance as *mut _)
        .check().unwrap();

    let instance_table =
        vkl::InstanceTable::load(instance, loader.get_instance_proc_addr);

    let physical_devices =
        vk::enumerate2!(instance_table, enumerate_physical_devices).unwrap();
    println!("physical_devices:");
    for &pdev in physical_devices.iter() {
        let mut props: vk::PhysicalDeviceProperties =
            std::mem::uninitialized();
        instance_table.get_physical_device_properties
            (pdev, &mut props as *mut _);
        println!("  - name: {:?}",
            ffi::CStr::from_ptr(&props.device_name as *const _));
        println!("    device_type: {}", match props.device_type {
            vk::PhysicalDeviceType::OTHER => "other",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "integrated_gpu",
            vk::PhysicalDeviceType::DISCRETE_GPU => "discrete_gpu",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "virtual_gpu",
            vk::PhysicalDeviceType::CPU => "cpu",
            _ => "unknown",
        });
        println!("    vendor_id: 0x{:x}", props.vendor_id);
        println!("    device_id: 0x{:x}", props.device_id);
        println!("    api_version: {}",
            Version::from(props.api_version));
        println!("    driver_version: {}",
            Version::from(props.driver_version));

        let qf_props = vk::enumerate2!(
            @void instance_table,
            get_physical_device_queue_family_properties,
            pdev,
        );
        println!("    queue_families:");
        for qf in qf_props.into_iter() {
            print!("      - queue_flags: ");
            print_bits! {
                qf.queue_flags;
                vk::QueueFlags::GRAPHICS_BIT => "GRAPHICS",
                vk::QueueFlags::COMPUTE_BIT => "COMPUTE",
                vk::QueueFlags::TRANSFER_BIT => "TRANSFER",
                vk::QueueFlags::SPARSE_BINDING_BIT => "SPARSE_BINDING",
                vk::QueueFlags::PROTECTED_BIT => "PROTECTED",
            };
            println!();

            println!("        queue_count: {}",
                qf.queue_count);
            println!("        timestamp_valid_bits: {}",
                qf.timestamp_valid_bits);
            println!(
                "        min_image_transfer_granularity: [{}, {}, {}]",
                qf.min_image_transfer_granularity.width,
                qf.min_image_transfer_granularity.height,
                qf.min_image_transfer_granularity.depth,
            );
        }

        let mut mem_props: vk::PhysicalDeviceMemoryProperties =
            std::mem::uninitialized();
        instance_table.get_physical_device_memory_properties
            (pdev, &mut mem_props as *mut _);

        let mem_types =
            &mem_props.memory_types[..mem_props.memory_type_count as usize];
        println!("    memory_types:");
        for &mem_type in mem_types.iter() {
            print!("      - property_flags: ");
            print_bits! {
                mem_type.property_flags;
                vk::MemoryPropertyFlags::DEVICE_LOCAL_BIT => "DEVICE_LOCAL",
                vk::MemoryPropertyFlags::HOST_VISIBLE_BIT => "HOST_VISIBLE",
                vk::MemoryPropertyFlags::HOST_COHERENT_BIT => "HOST_COHERENT",
                vk::MemoryPropertyFlags::HOST_CACHED_BIT => "HOST_CACHED",
                vk::MemoryPropertyFlags::LAZILY_ALLOCATED_BIT =>
                    "LAZILY_ALLOCATED",
                vk::MemoryPropertyFlags::PROTECTED_BIT => "PROTECTED",
            };
            println!();
            println!("        heap_index: {}", mem_type.heap_index);
        }

        let mem_heaps =
            &mem_props.memory_heaps[..mem_props.memory_heap_count as usize];
        println!("    memory_heaps:");
        for &mem_heap in mem_heaps.iter() {
            println!("      - size: {} GiB",
                mem_heap.size as f32 / ((2u64 << 30) as f32));
            print!("        flags: ");
            print_bits! {
                mem_heap.flags;
                vk::MemoryHeapFlags::DEVICE_LOCAL_BIT => "DEVICE_LOCAL",
                vk::MemoryHeapFlags::MULTI_INSTANCE_BIT => "MULTI_INSTANCE",
            };
            println!();
        }

        let exts = vk::enumerate2!(
            instance_table,
            enumerate_device_extension_properties,
            pdev,
            ptr::null(),
        ).unwrap();
        println!("    extensions:");
        for ext in exts.into_iter() {
            println!("      - name: {:?}",
                ffi::CStr::from_ptr(&ext.extension_name as *const _));
            println!("        spec_version: {}",
                Version::from(ext.spec_version));
        }
    }

    instance_table.destroy_instance(ptr::null());

    // Make sure the loader library is not unloaded before this point.
    std::mem::drop(loader);
}
