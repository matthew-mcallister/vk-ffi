# vk-ffi

## Motivation

This crate provides comprehensive, automatically generated bindings to
the Vulkan core API along with cross-platform extensions. The resulting
bindings are:

* **Complete**: Every command, constant, and struct is exposed for all
  core API versions and all supported extensions.
* **Up-to-date**: The bindings are (or may be) updated with every
  revision to the API.
* **Portable**: The Vulkan core API is cross-platform and
  backwards-compatible by design.
* **Stable, maintainable**: Frameworks and libraries come and go, but
  the Vulkan API proper is designed for forward and backward
  compatibility.
* **Interoperable**: Binary interop with code written to the Vulkan API
  in any language is trivial, and loaders, frameworks, libraries, and
  tutorials written in Rust may choose to target these particular
  bindings to better integrate with the ecosystem at large.

## Contents

The top-level module includes every constant, type, and function pointer
definition from `vulkan_core.h` (some preprocessor defines excluded).
Names have been transformed to fit the Rust idiom, but definitions are
otherwise unchanged.

Global function prototypes are not exported, as they are better left a
responsibility of loader libraries.

Definitions from the platform headers are TBD; the current expectation
is for applications to integrate with GLFW or equivalent. The Windows
headers may take a little extra work.

Some Rust macros are also exported, including Rust-specific convenience
macros.
