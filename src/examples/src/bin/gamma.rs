// Vulkan compute demonstration which produces a plot of the gamma
// function in a rectangular region (with Re(z) > 1/2).
#![feature(try_blocks)]

use std::ffi::c_void;
use std::io::Write;
use std::ptr;

use examples::*;

const GAMMA_SPV_BYTES: &'static [u8] =
    include_bytes!(data_file!("gamma/gamma.spv"));

const IMAGE_DIMS: [u32; 2] = [401, 401];

fn write_tga_header<W: Write>(mut w: W) {
    // All values little-endian
    w.write_all(&[
        0,              // ID length
        0,              // Colormap type
        2,              // Image type = uncompressed true-color
        0, 0, 0, 0, 0,  // Colormap spec (unused)
        0, 0, 0, 0,     // X and Y origins (unused)
        // Image dimensions in pixels
        IMAGE_DIMS[0] as u8, (IMAGE_DIMS[0] >> 8) as u8,
        IMAGE_DIMS[1] as u8, (IMAGE_DIMS[1] >> 8) as u8,
        24,             // Pixel-depth
        0,              // Image descriptor (unused)
    ]).unwrap();
}

fn rgb32_to_bgr8([r, g, b]: [f32; 3]) -> [u8; 3] {
    [(b * 255.0 + 0.5) as u8, (g * 255.0 + 0.5) as u8, (r * 255.0 + 0.5) as u8]
}

// An aesthetically pleasing but inaccurate conversion
// s, l are in [0, 1]; h is in radians
fn hsl_to_rgb([h, s, l]: [f32; 3]) -> [f32; 3] {
    let h = h / (2.0 * std::f32::consts::PI);
    let h = if h < 0.0 { 1.0 + h } else { h };
    let h = h % 1.0;
    let n = (6.0 * h) as u32;
    let x = (6.0 * h) % 1.0;
    let t = f32::sin(0.5 * std::f32::consts::PI * x);
    let t = t * t;
    let [r, g, b] = match n {
        0 => [1.0, t, 0.0],
        1 => [1.0 - t, 1.0, 0.0],
        2 => [0.0, 1.0, t],
        3 => [0.0, 1.0 - t, 1.0],
        4 => [t, 0.0, 1.0],
        5 => [1.0, 0.0, 1.0 - t],
        _ => unreachable!(),
    };
    let c = s * (1.0 - (2.0 * l - 1.0).abs());
    let m = l - 0.5 * c;
    [r * c + m, g * c + m, b * c + m]
}

fn main() {
    unsafe { unsafe_main() }
}

unsafe fn unsafe_main() {
    let sys = VulkanSys::new();

    // Create compute pipeline
    let create_info = vk::ShaderModuleCreateInfo {
        code_size: GAMMA_SPV_BYTES.len(),
        p_code: from_bytes_le(GAMMA_SPV_BYTES).as_ptr(),
        ..Default::default()
    };
    let mut shader_mod = vk::null();
    sys.device.create_shader_module(&create_info, ptr::null(), &mut shader_mod)
        .check().unwrap();

    let binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::COMPUTE_BIT,
        p_immutable_samplers: ptr::null(),
    };
    let create_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: 1,
        p_bindings: &binding,
        ..Default::default()
    };
    let mut set_layout = vk::null();
    sys.device.create_descriptor_set_layout
        (&create_info, ptr::null(), &mut set_layout)
        .check().unwrap();

    let create_info = vk::PipelineLayoutCreateInfo {
        set_layout_count: 1,
        p_set_layouts: &set_layout,
        ..Default::default()
    };
    let mut layout = vk::null();
    sys.device.create_pipeline_layout
        (&create_info, ptr::null(), &mut layout)
        .check().unwrap();

    let stage_create_info = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::COMPUTE_BIT,
        p_name: c_str!("main"),
        module: shader_mod,
        p_specialization_info: ptr::null(),
        ..Default::default()
    };
    let create_info = vk::ComputePipelineCreateInfo {
        stage: stage_create_info,
        layout,
        ..Default::default()
    };
    let mut pipeline = vk::null();
    sys.device.create_compute_pipelines(
        vk::null(),
        1,
        &create_info,
        ptr::null(),
        &mut pipeline,
    ).check().unwrap();

    // Create and bind storage buffer
    let mut props: vk::PhysicalDeviceMemoryProperties = Default::default();
    sys.instance.get_physical_device_memory_properties
        (sys.physical_device, &mut props);
    let mem_type = props.memory_types[..props.memory_type_count as usize]
        .iter()
        .position(|ty| ty.property_flags.intersects(
            vk::MemoryPropertyFlags::HOST_VISIBLE_BIT
        ))
        .unwrap() as u32;

    let num_elems = IMAGE_DIMS[0] as usize * IMAGE_DIMS[1] as usize;

    let buf_size = (num_elems * std::mem::size_of::<[f32; 2]>()) as u64;
    let allocate_info = vk::MemoryAllocateInfo {
        allocation_size: buf_size,
        memory_type_index: mem_type,
        ..Default::default()
    };
    let mut buf_mem = vk::null();
    sys.device.allocate_memory
        (&allocate_info, ptr::null(), &mut buf_mem)
        .check().unwrap();

    let create_info = vk::BufferCreateInfo {
        size: buf_size,
        usage: vk::BufferUsageFlags::STORAGE_BUFFER_BIT,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let mut buffer = vk::null();
    sys.device.create_buffer
        (&create_info, ptr::null(), &mut buffer)
        .check().unwrap();

    sys.device.bind_buffer_memory(buffer, buf_mem, 0).check().unwrap();

    // Create and update descriptor set
    let pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
    };
    let create_info = vk::DescriptorPoolCreateInfo {
        max_sets: 1,
        pool_size_count: 1,
        p_pool_sizes: &pool_size,
        ..Default::default()
    };
    let mut desc_pool = vk::null();
    sys.device.create_descriptor_pool
        (&create_info, ptr::null(), &mut desc_pool)
        .check().unwrap();

    let allocate_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool: desc_pool,
        descriptor_set_count: 1,
        p_set_layouts: &set_layout,
        ..Default::default()
    };
    let mut desc_set = vk::null();
    sys.device.allocate_descriptor_sets
        (&allocate_info, &mut desc_set)
        .check().unwrap();

    let desc_buf_info = vk::DescriptorBufferInfo {
        buffer,
        offset: 0,
        range: vk::WHOLE_SIZE,
    };
    let desc_write = vk::WriteDescriptorSet {
        dst_set: desc_set,
        dst_binding: 0,
        dst_array_element: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
        p_buffer_info: &desc_buf_info,
        ..Default::default()
    };
    sys.device.update_descriptor_sets
        (1, &desc_write, 0, ptr::null());

    // Create and record to command buffer
    let create_info = Default::default();
    let mut command_pool = vk::null();
    sys.device.create_command_pool
        (&create_info, ptr::null(), &mut command_pool)
        .check().unwrap();

    let allocate_info = vk::CommandBufferAllocateInfo {
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: 1,
        ..Default::default()
    };
    let mut cmd_buf = vk::null();
    sys.device.allocate_command_buffers
        (&allocate_info, &mut cmd_buf)
        .check().unwrap();

    let begin_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT_BIT,
        ..Default::default()
    };
    sys.device.begin_command_buffer(cmd_buf, &begin_info)
        .check().unwrap();
    sys.device.cmd_bind_pipeline
        (cmd_buf, vk::PipelineBindPoint::COMPUTE, pipeline);
    sys.device.cmd_bind_descriptor_sets(
        cmd_buf,
        vk::PipelineBindPoint::COMPUTE,
        layout,
        0,
        1,
        &desc_set,
        0,
        ptr::null(),
    );
    sys.device.cmd_dispatch(cmd_buf, IMAGE_DIMS[0], IMAGE_DIMS[1], 1);
    sys.device.end_command_buffer(cmd_buf).check().unwrap();

    // Submit
    let submit_info = vk::SubmitInfo {
        command_buffer_count: 1,
        p_command_buffers: &cmd_buf,
        ..Default::default()
    };
    sys.device.queue_submit(sys.queue, 1, &submit_info, vk::null())
        .check().unwrap();

    sys.device.device_wait_idle().check().unwrap();

    // Write results to image
    let mut data: *mut c_void = ptr::null_mut();
    sys.device.map_memory(
        buf_mem,
        0,
        vk::WHOLE_SIZE as u64,
        Default::default(),
        &mut data,
    ).check().unwrap();
    let data: &'static [[f32; 2]] =
        std::slice::from_raw_parts(data as _, num_elems);

    let out_path = output_file!("gamma.tga");
    let file = std::fs::OpenOptions::new()
        .write(true).truncate(true).create(true)
        .open(out_path)
        .unwrap();
    let mut buffered = std::io::BufWriter::new(file);

    // Plot the output as a contour plot with hue as complex argument
    write_tga_header(&mut buffered);
    for &z in data.iter() {
        let [x, y] = z;
        let h = f32::atan2(z[1], z[0]);
        let mag = (x * x + y * y).sqrt();
        let l = 0.2 * (mag % 1.0) + 0.35;
        let c = rgb32_to_bgr8(hsl_to_rgb([h, 1.0, l]));
        buffered.write_all(&c).unwrap();
    }

    // Clean up
    sys.device.unmap_memory(buf_mem);

    sys.device.destroy_command_pool(command_pool, ptr::null());
    sys.device.destroy_descriptor_pool(desc_pool, ptr::null());

    sys.device.destroy_buffer(buffer, ptr::null());
    sys.device.free_memory(buf_mem, ptr::null());

    sys.device.destroy_pipeline(pipeline, ptr::null());
    sys.device.destroy_pipeline_layout(layout, ptr::null());
    sys.device.destroy_descriptor_set_layout(set_layout, ptr::null());
    sys.device.destroy_shader_module(shader_mod, ptr::null());
}
