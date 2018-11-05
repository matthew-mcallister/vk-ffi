# vk-ffi-loader

This crate provides an API for loading Vulkan commands using a loader
library. It is agnostic to how you link to the loader: at compile-time, at
run-time, or through an intermediate library such as GLFW.

The Vulkan API is broken up into many smaller APIs and corresponding function
pointer tables are defined covering core functionality for versions 1.0 and 1.1
as well as extensions. Each API is either an instance-level API or a
device-level API and the corresponding function pointer table's `load` method
takes `VkInstance` and `PFN_vkGetInstanceProcAddr` or `VkDevice` and
`PFN_vkGetDeviceProcAddr` parameters, respectively.

Extension function tables are located in the `extensions` module, while core
interfaces are in the `v1_0` and `v1_1` modules as `CoreInstance` and
`CoreDevice`. The latter two modules re-export everything in `extensions`, so
you can use a single import statement like so:
```
use vk_ffi_loader::v1_1 as vk_loader;
```

Each function table also stores the `VkInstance` or `VkDevice` parameter it was
loaded with and defines methods which automatically pass the handle to the
underlying command when appropriate. Thus, you get the safer and more ergonomic
```
instance_table.destroy_instance();
```
instead of
```
(instance_table.destroy_instance)(instance);
```
if calling the stored function pointer directly.
