# vk-ffi

## Features

This crate provides comprehensive, automatically generated bindings to
the Vulkan core API along with cross-platform extensions. The resulting
bindings are:

* **Complete**: Every command, constant, and struct is exposed for all
  core API versions and all supported extensions.
* **Up-to-date**: New bindings can be generated with every revision to
  the API.
* **Portable**: Platform specifics aren't baked into the bindings, such
  as requiring a specific loader method or library.
* **Interoperable**: These bindings are binary compatible with C
  libraries (such as AMD's `VulkanMemoryAllocator`) and with any choice
  of loader library.

## Contents

The top-level module includes all data type definitions and aliases ---
everything with the `Vk*` prefix. The `pfn` submodule includes all
function pointer definitions, so everything with the `PFN_vk` prefix.
Struct/enum members and function arguments have been renamed to be more
idiomatic, but definitions are not changed. Bitmasks and enums have been
made into proper types, improving type safety.

Some traits are implemented out of the box, such as `Default`, but
more specific traits can be enabled with the `extra-traits` feature,
e.g. `FromStr` for enums and bitmasks.

Some Rust macros are exported, including translations of C macros
defined by the standard as well as Rust-specific convenience macros.
