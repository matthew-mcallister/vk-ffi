# vulkan-loader

This crate provides an API for loading Vulkan commands using a loader
library. It is agnostic to how you link to the loader: linking to the
library at compile time or loading it at runtime are both possible. It
is also agnostic to API version and supports almost all extensions, so
it is usable with older implementations.

Given a pointer to `vkGetInstanceProcAddr` or `vkGetDeviceProcAddr`, you
can load a method table containing all instance- or device-level
commands exposed by the implementation. This table stores the
`VkInstance` or `VkDevice` parameter it was loaded with and defines
methods which automatically pass the handle to the underlying command
when appropriate. Thus, you get
```
device_table.destroy_device(...);
```
instead of
```
(device_table.pfn_destroy_device)(device, ...);
```
when calling the stored function pointer directly.

## Caveats

This library doesn't do any validation to make sure that the extensions
you attempt to use were actually enabled. Unavailable function pointers
will be set to `null`.
