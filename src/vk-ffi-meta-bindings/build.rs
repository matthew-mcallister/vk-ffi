#![feature(box_patterns)]
#![feature(crate_visibility_modifier)]
#![feature(nll)]
#![feature(try_blocks)]
#![feature(uniform_paths)]

#![recursion_limit = "256"]

extern crate bindgen;
extern crate heck;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

#[macro_use]
mod build_src;

use std::env;
use std::io::Write;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;

use self::build_src::{emit, rewrite, scrape};

const VK_HEADER_DIR: &'static str = "vendor/Vulkan-Docs/include/vulkan";

fn main() {
    let out_dir: PathBuf = env::var_os("OUT_DIR").unwrap().into();
    println!("out_dir: {:?}", &out_dir);

    // TODO: Build should depend on build_src/*.rs and VK_HEADER_DIR/*.h
    println!("cargo:rerun-if-changed=build_src/mod.rs");
    println!("cargo:rerun-if-changed=build_src/emit.rs");
    println!("cargo:rerun-if-changed=build_src/rewrite.rs");
    println!("cargo:rerun-if-changed=build_src/scrape.rs");

    let raw = generate_raw_bindings();

    let tokens = std::str::FromStr::from_str(&raw).unwrap();
    let tokens = rewrite::rewrite_tokens(tokens);

    let mut ast = syn::parse2(tokens).unwrap();
    rewrite::rewrite_ast(&mut ast);

    let defs = scrape::parse_file(ast);
    let emitted = emit::emit_defs(defs);

    for emit::EmittedFile { file_name, tokens } in emitted {
        let path = out_dir.join(&file_name);
        let mut file = open_output(&path);
        write_defs(&mut file, tokens);
    }
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

fn open_output(out: &Path) -> File {
    OpenOptions::new().write(true).create(true).truncate(true).open(out)
        .unwrap()
}

fn write_defs<W: Write>(mut w: W, tokens: TokenStream) {
    write!(w, "{}", tokens).unwrap();
}
