#![feature(crate_visibility_modifier)]
#![feature(try_blocks)]

extern crate bindgen;
extern crate heck;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod build_src;

use std::env;
use std::io::Write;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use quote::ToTokens;

use self::build_src::global_rename;

const VK_HEADER_DIR: &'static str = "vendor/Vulkan-Docs/include/vulkan";

fn output_file() -> PathBuf {
    let mut path: PathBuf = env::var_os("OUT_DIR").unwrap().into();
    path.push("bindings.rs");
    path
}

fn dev_mode() -> bool { env::var_os("DEV_MODE").is_some() }

fn main() {
    let out = output_file();
    println!("output_file: {:?}", &out);
    println!("cargo:rerun-if-changed=build.h");

    let bindings = generate_raw_bindings();
    let bindings = rename_items(&bindings[..]);

    let mut file = open_output(&out);
    write_bindings(&mut file, bindings);
}

fn generate_raw_bindings() -> String {
    println!("cargo:rerun-if-changed=stub.h");
    println!("cargo:rerun-if-changed={}/{}", VK_HEADER_DIR, "vulkan_core.h");
    println!("cargo:rerun-if-changed={}/{}", VK_HEADER_DIR, "vk_platform.h");
    bindgen::builder()
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
        .layout_tests(dev_mode())
        .rustfmt_bindings(false)
        .generate()
        .unwrap()
        .to_string()
}

// Find-and-replace on identifiers to implement Rust's naming scheme.
fn rename_items(bindings: &str) -> TokenStream {
    let tokens = std::str::FromStr::from_str(bindings).unwrap();
    global_rename::do_rename(tokens)
}

fn open_output(out: &Path) -> File {
    OpenOptions::new().write(true).create(true).truncate(true).open(out)
        .unwrap()
}

fn write_bindings<W: Write>(mut w: W, bindings: TokenStream) {
    // Write one item per line for more readable error messages
    let ast: syn::File = syn::parse2(bindings).unwrap();
    for attr in ast.attrs.into_iter() {
        writeln!(w, "{}", attr.into_token_stream()).unwrap();
    }
    for item in ast.items.into_iter() {
        writeln!(w, "{}", item.into_token_stream()).unwrap();
    }
}
