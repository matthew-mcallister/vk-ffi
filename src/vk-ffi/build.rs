#![feature(crate_visibility_modifier)]
#![feature(try_blocks)]
#![feature(uniform_paths)]

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

use self::build_src::enum_rename;
use self::build_src::global_rename;

fn split_prefix<'a>(s: &'a str, prefix: &str) -> Option<(&'a str, &'a str)> {
    if s.starts_with(prefix) { Some(s.split_at(prefix.len())) }
    else { None }
}

fn strip_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    Some(split_prefix(s, prefix)?.1)
}

fn split_suffix<'a>(s: &'a str, suffix: &str) -> Option<(&'a str, &'a str)> {
    if s.ends_with(suffix) { Some(s.split_at(s.len() - suffix.len())) }
    else { None }
}

fn strip_suffix<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
    Some(split_suffix(s, suffix)?.0)
}

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
    let bindings = enforce_rust_naming(bindings);
    let bindings = strip_enum_prefixes(bindings);

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
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .prepend_enum_name(false)
        .impl_debug(true)
        .layout_tests(dev_mode())
        .rustfmt_bindings(false)
        .generate()
        .unwrap()
        .to_string()
}

// Find-and-replace on identifiers to implement Rust's naming scheme.
fn enforce_rust_naming(bindings: String) -> TokenStream {
    let tokens = std::str::FromStr::from_str(&bindings).unwrap();
    global_rename::do_rename(tokens)
}

// Enums are namespaced, so prefixes are unnecessary.
fn strip_enum_prefixes(bindings: TokenStream) -> syn::File {
    let mut ast: syn::File = syn::parse2(bindings).unwrap();
    enum_rename::do_rename(&mut ast);
    ast
}

fn open_output(out: &Path) -> File {
    OpenOptions::new().write(true).create(true).truncate(true).open(out)
        .unwrap()
}

fn write_bindings<W: Write>(mut w: W, ast: syn::File) {
    // Write one item per line for more readable error messages
    for attr in ast.attrs.into_iter() {
        writeln!(w, "{}", attr.into_token_stream()).unwrap();
    }
    for item in ast.items.into_iter() {
        writeln!(w, "{}", item.into_token_stream()).unwrap();
    }
}
