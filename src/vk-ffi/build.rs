extern crate bindgen;

use std::env;
use std::error::Error;
use std::path::PathBuf;

const VK_HEADER_DIR: &'static str = "vendor/Vulkan-Docs/include/vulkan";

fn main() -> Result<(), Box<Error>> {
    let bindings = generate_raw_bindings()?;
    let out_dir: PathBuf = env::var_os("OUT_DIR").unwrap().into();
    let bindings_path = out_dir.join("bindings.rs");
    std::fs::write(bindings_path, bindings)?;
    Ok(())
}

fn generate_raw_bindings() -> Result<String, Box<Error>> {
    println!("cargo:rerun-if-changed=stub.h");
    println!("cargo:rerun-if-changed={}/{}", VK_HEADER_DIR, "vulkan_core.h");
    println!("cargo:rerun-if-changed={}/{}", VK_HEADER_DIR, "vk_platform.h");
    let res = bindgen::builder()
        .header("stub.h")
        .whitelist_type("Vk.*")
        .whitelist_var("VK_.*")
        // See github.com/rust-lang-nursery/rust-bindgen/issues/1223
        //.blacklist_item("__.*")
        .blacklist_item("VK_VERSION_.*")
        .blacklist_item("VK_HEADER_VERSION")
        .blacklist_item("VK_.*[a-z].*") // Extension #defines
        .default_enum_style(bindgen::EnumVariation::Consts)
        .prepend_enum_name(false)
        .impl_debug(true)
        .generate()
        .map_err(|_| "failed to generate bindings")?
        .to_string();
    Ok(res)
}
