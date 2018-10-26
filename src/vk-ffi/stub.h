#define VK_NO_PROTOTYPES

typedef long long NonDispatchableHandleVkFfi;
#define VK_DEFINE_NON_DISPATCHABLE_HANDLE(object) \
    typedef NonDispatchableHandleVkFfi object;

#include "vendor/Vulkan-Docs/include/vulkan/vulkan_core.h"
