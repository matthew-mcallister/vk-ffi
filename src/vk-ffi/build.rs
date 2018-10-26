#![feature(box_patterns)]
#![feature(crate_visibility_modifier)]
#![feature(try_blocks)]
#![feature(uniform_paths)]

extern crate bindgen;
extern crate heck;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod build_src;

use std::env;
use std::io::Write;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use quote::ToTokens;

use self::build_src::{enum_rewrite, global_rename, handle_rewrite};

const VK_HEADER_DIR: &'static str = "vendor/Vulkan-Docs/include/vulkan";

fn output_file() -> PathBuf {
    let mut path: PathBuf = env::var_os("OUT_DIR").unwrap().into();
    path.push("bindings.rs");
    path
}

fn main() {
    let out = output_file();
    println!("output_file: {:?}", &out);
    println!("cargo:rerun-if-changed=build.h");

    let raw = generate_raw_bindings();

    let tokens = std::str::FromStr::from_str(&raw).unwrap();
    let tokens = global_rename::do_rename(tokens);

    let mut ast = syn::parse2(tokens).unwrap();
    skip_compiler_types(&mut ast);
    enum_rewrite::do_rewrite(&mut ast);
    handle_rewrite::do_rewrite(&mut ast);

    let mut file = open_output(&out);
    write_bindings(&mut file, ast);
}

fn generate_raw_bindings() -> String {
    println!("cargo:rerun-if-changed=stub.h");
    println!("cargo:rerun-if-changed={}/{}", VK_HEADER_DIR, "vulkan_core.h");
    println!("cargo:rerun-if-changed={}/{}", VK_HEADER_DIR, "vk_platform.h");
    bindgen::builder()
        .header("stub.h")
        .whitelist_type("Vk.*")
        .whitelist_type("PFN_.*")
        .whitelist_var("VK_.*")
        // See github.com/rust-lang-nursery/rust-bindgen/issues/1223
        //.blacklist_item("__.*")
        .blacklist_item("NonDispatchableHandleVkFfi")
        .blacklist_item("VK_VERSION_.*")
        .blacklist_item("VK_HEADER_VERSION")
        .blacklist_item("VK_NULL_HANDLE")
        .blacklist_item("VK_.*[a-z].*") // Extension #defines
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .prepend_enum_name(false)
        .layout_tests(false)
        .rustfmt_bindings(false)
        .generate()
        .unwrap()
        .to_string()
}

// Get rid of __int8_t etc.
fn skip_compiler_types(ast: &mut syn::File) {
    ast.items.retain(|item| {
        if let syn::Item::Type(ref ty_def) = item {
            !ty_def.ident.to_string().starts_with("__")
        } else { true }
    });
}

fn open_output(out: &Path) -> File {
    OpenOptions::new().write(true).create(true).truncate(true).open(out)
        .unwrap()
}

fn write_bindings<W: Write>(mut w: W, ast: syn::File) {
    // Write one item per line for more readable error messages
    for item in ast.items.into_iter() {
        writeln!(w, "{}", item.into_token_stream()).unwrap();
    }
}
