/// Basically just tests that std:;fmt::Debug is implemented and works.

use std::ptr;

use examples::*;

fn main() {
    println!("{:#?}", vk::ApplicationInfo {
        p_application_name: c_str!("debug example"),
        application_version: vk::make_version!(0, 1, 0),
        p_engine_name: ptr::null(),
        engine_version: vk::make_version!(0, 1, 0),
        api_version: vk::API_VERSION_1_0,
        ..Default::default()
    });

    unsafe {
        use std::mem::transmute;
        println!("{:#?}", vk::AllocationCallbacks {
            p_user_data: 123456789 as _,
            pfn_allocation: transmute(987654321usize),
            pfn_reallocation: transmute(123456789usize),
            pfn_free: transmute(987654321usize),
            pfn_internal_allocation: transmute(123456789usize),
            pfn_internal_free: transmute(987654321usize),
        });
    }

    println!("{:#?}", vk::ShaderStageFlags::VERTEX_BIT);

    let color = vk::ClearColorValue { float_32: [0.0; 4] };
    println!("{:#?}", color);
    println!("{:#?}", vk::ClearValue { color });

    println!("{:#?}", vk::BufferImageCopy {
        buffer_offset: 1234,
        buffer_row_length: 100,
        buffer_image_height: 200,
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR_BIT,
            mip_level: 1,
            base_array_layer: 2,
            layer_count: 3,
        },
        image_offset: vk::Offset3D::new(1, 2, 3),
        image_extent: vk::Extent3D::new(1, 2, 3),
    });

    println!("{:?}", vk::ExtensionProperties {
        extension_name: [0; 256],
        spec_version: 1414,
    });
}
