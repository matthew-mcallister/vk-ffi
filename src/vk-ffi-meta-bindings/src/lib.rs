#![recursion_limit = "8192"]

extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use vk_ffi_meta_defs::*;

include!(concat!(env!("OUT_DIR"), "/consts.rs"));
include!(concat!(env!("OUT_DIR"), "/enums.rs"));
include!(concat!(env!("OUT_DIR"), "/fn_pointers.rs"));
include!(concat!(env!("OUT_DIR"), "/handles.rs"));
include!(concat!(env!("OUT_DIR"), "/structs.rs"));
include!(concat!(env!("OUT_DIR"), "/type_aliases.rs"));
include!(concat!(env!("OUT_DIR"), "/unions.rs"));
