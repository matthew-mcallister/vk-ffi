#[macro_export]
macro_rules! vk_make_version {
        ($major:expr, $minor:expr, $patch:expr) => {
        ($major << 22) | ($minor << 12) | $patch
    }
}

#[macro_export]
macro_rules! vk_version_major {
    ($version:expr) => { $version >> 22 }
}

#[macro_export]
macro_rules! vk_version_minor {
    ($version:expr) => { ($version >> 12) & 0x3ff }
}

#[macro_export]
macro_rules! vk_version_patch {
    ($version:expr) => { $version & 0xfff }
}

/// Converts a `VkResult` to a `Result<VkResult, VkResult>`, branching
/// on whether the code signifies an error.
#[macro_export]
macro_rules! vk_check {
    ($res:expr) => { if $res.is_success() { Ok($res) } else { Err($res) } }
}

/// Handles the boilerplate of making two calls to `VkEnumerate*`: one
/// to get the number of elements, and another to fill the array. This
/// macro yields `Result<Vec<_>, VkResult>`, and never returns
/// VK_INCOMPLETE.
///
/// The macro can take a second argument, which will be passed to the
/// API call.
///
/// The macro is overloaded when the first argument is of the form
/// `var.method`, so that the call is treated as a method invocation
/// instead of a call to a callable struct member. To get the latter
/// behavior, apply parentheses, as in `(var.method)`.
///
/// By prepending `@void`, this macro can also be used on commands like
/// `vkGetPhysicalDeviceProperties`, which return `void` instead of
/// `VkResult`. In this case, it just returns `Vec<_>`.
#[macro_export]
macro_rules! vk_enumerate {
    // Regular versions
    ($table:ident.$method:ident) => {
        vk_enumerate!(@impl ($table.$method) ())
    };
    ($table:ident.$method:ident, $object:expr) => {
        vk_enumerate!(@impl ($table.$method) ($object,))
    };
    ($command:expr) => {
        vk_enumerate!(@impl ($command) ())
    };
    ($command:expr, $object:expr) => {
        vk_enumerate!(@impl ($command) ($object,))
    };
    (@impl ($($command:tt)*) ($($object:tt)*)) => {{
        let x: ::std::result::Result<_, $crate::Result> = try {
            let mut n: u32 = 0;
            $($command)*($($object)* &mut n as *mut _, ::std::ptr::null_mut())
                .check()?;
            let mut v = ::std::vec::Vec::with_capacity(n as usize);
            $($command)*($($object)* &mut n as *mut _, v.as_mut_ptr())
                .check()?;
            v.set_len(n as usize);
            v
        };
        x
    }};
    // Void versions
    (@void $table:ident.$method:ident) => {
        vk_enumerate!(@impl @void ($table.$method) ())
    };
    (@void $table:ident.$method:ident, $object:expr) => {
        vk_enumerate!(@impl @void ($table.$method) ($object,))
    };
    (@void $command:expr) => {
        vk_enumerate!(@impl @void ($command) ())
    };
    (@void $command:expr, $object:expr) => {
        vk_enumerate!(@impl @void ($command) ($object,))
    };
    (@impl @void ($($command:tt)*) ($($object:tt)*)) => {{
        let mut n: u32 = 0;
        $($command)*($($object)* &mut n as *mut _, ::std::ptr::null_mut());
        let mut v = ::std::vec::Vec::with_capacity(n as usize);
        $($command)*($($object)* &mut n as *mut _, v.as_mut_ptr());
        v.set_len(n as usize);
        v
    }};
}
