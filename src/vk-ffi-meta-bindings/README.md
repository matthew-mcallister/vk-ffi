# vk-ffi-meta-bindings

This is a "meta-crate" which exposes the struct/function/etc.
definitions used to generate Vulkan API bindings.

This crate provides all the definitions in the Vulkan API as Rust
objects from the `vk-ffi-meta-defs` crate, e.g. `Enum`, `Handle`,
`Alias`. These can be used to generate bindings using the `syn` library
in the same way procedural macros are created.

The definitions themselves are all produced at compile time using
`bindgen` and baked into the resulting library; this saves on build time
for dependent crates.
