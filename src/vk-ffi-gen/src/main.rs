#![feature(box_patterns)]
#![feature(crate_visibility_modifier)]
#![feature(try_blocks)]
#![feature(uniform_paths)]

extern crate bindgen;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_more;
extern crate heck;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use std::io::Write;
use std::fs::OpenOptions;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};

macro_rules! get_variant {
    ($var:path, $val:expr) => {
        if let $var(inner) = $val { Some(inner) } else { None }
    }
}

mod defs;
mod emit;
mod rewrite;
mod scrape;

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

fn map_ident(ident: &syn::Ident, f: impl FnOnce(String) -> String) ->
    syn::Ident
{
    syn::Ident::new(&f(ident.to_string()), ident.span())
}

fn main() {
    let matches = clap_app!(("vk-ffi-meta-gen") =>
        (version: "0.1")
        (about: "Generate Vulkan meta bindings")
        (@arg INPUT_DIR: -i --input +takes_value +required
            "Path to Vulkan-Docs repo")
        (@arg OUTPUT_DIR: -o --output +takes_value +required
            "Path to output directory")
    ).get_matches();

    let in_dir: &Path = Path::new(matches.value_of_os("INPUT_DIR").unwrap());
    let out_dir: &Path = Path::new(matches.value_of_os("OUTPUT_DIR").unwrap());

    let raw = generate_raw_bindings(in_dir);

    let tokens = std::str::FromStr::from_str(&raw).unwrap();
    let tokens = rewrite::rewrite_tokens(tokens);

    let mut ast = syn::parse2(tokens).unwrap();
    rewrite::rewrite_ast(&mut ast);

    let defs = scrape::parse_file(ast);

    let bindings = emit::bindings::emit(&defs);
    let bindings_path = out_dir.join("bindings.rs");
    let mut bindings_file = OpenOptions::new()
        .write(true).create(true).truncate(true).open(bindings_path).unwrap();
    write_tokens(&mut bindings_file, bindings);
}

const STUB_HEADER: &'static str = r#"
    #define VK_NO_PROTOTYPES

    typedef long long NonDispatchableHandleVkFfi;
    #define VK_DEFINE_NON_DISPATCHABLE_HANDLE(object) \
        typedef NonDispatchableHandleVkFfi object;

    #include "vulkan/vulkan_core.h"
"#;

fn generate_raw_bindings(in_dir: &Path) -> String {
    let include_dir = std::fs::canonicalize(in_dir.join("include")).unwrap();
    bindgen::builder()
        .header_contents("stub.h", STUB_HEADER)
        .clang_arg(format!("-I{}", include_dir.to_str().unwrap()))
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

fn write_tokens<W: Write>(mut out: W, tokens: TokenStream) {
    for tt in tokens.into_iter() {
        match tt {
            TokenTree::Group(ref grp) => match grp.delimiter() {
                proc_macro2::Delimiter::Bracket |
                proc_macro2::Delimiter::Brace =>
                    writeln!(out, "{}", tt).unwrap(),
                _ => write!(out, "{}", tt).unwrap(),
            },
            TokenTree::Punct(ref punct) => match punct.as_char() {
                ';' => writeln!(out, ";").unwrap(),
                _ => write!(out, "{}", tt).unwrap(),
            },
            _ => write!(out, "{} ", tt).unwrap(),
        }
    }
}
