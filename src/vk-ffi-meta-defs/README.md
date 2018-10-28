# vk-ffi-meta-defs

This is a "meta-crate" which houses definitions used to generate various
Vulkan API bindings.

This crate includes definitions like `Enum`, `Handle`, `Alias`, and
`FunctionPointer`, which are furnished with syntax tree snippets (from
the `syn` crate) and other info as needed to produce Rust bindings.
Metadata/annotations from the Vulkan API registry are not included.

Each definition has an associated format for serializing it to and from
a `TokenStream`. This is not intended for code generation purposes, but
only as a crude serialization technique used by `vk-ffi-meta-bindings`.
