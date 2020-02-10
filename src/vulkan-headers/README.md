# vulkan-headers

## Features

This crate provides comprehensive, automatically generated bindings to
the Vulkan core API along with all extensions (except platform-specific
WSI extensions). The bindings include all definitions from the current
core API and almost all registered extensions. An optional function
loader is provided as a separate crate, `vulkan-headers`.

## Contents

The top-level module includes all data type definitions and aliases ---
everything with the `Vk*` prefix. The `pfn` submodule includes all
function pointer definitions, meaning all definitions with the `PFN_vk`
prefix. Struct/enum members and function arguments have been renamed to
be more idiomatic. Bitmasks and enums are type safe.

Some traits are implemented out of the box, such as `Default` and
`BitAnd`/`Or`/`Xor` for bitmasks. Some handy traits are implemented when
the `reflection` feature is enabled, such as `FromStr` for enums and
bitmasks.

Some Rust macros are exported, including implementations of C macros
defined by the standard as well as new convenience macros.
