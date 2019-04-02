#[macro_export]
macro_rules! make_version {
        ($major:expr, $minor:expr, $patch:expr) => {
        ($major << 22) | ($minor << 12) | $patch
    }
}

#[macro_export]
macro_rules! version_major {
    ($version:expr) => { $version >> 22 }
}

#[macro_export]
macro_rules! version_minor {
    ($version:expr) => { ($version >> 12) & 0x3ff }
}

#[macro_export]
macro_rules! version_patch {
    ($version:expr) => { $version & 0xfff }
}

/// Converts a `VkResult` to a `Result<VkResult, VkResult>`, branching
/// on whether the code signifies an error.
#[macro_export]
macro_rules! check {
    ($res:expr) => { if $res.is_success() { Ok($res) } else { Err($res) } }
}

/// Handles the boilerplate of making two calls to `VkEnumerate*`: one
/// to get the number of elements, and another to fill the array. This
/// macro yields `Result<Vec<_>, VkResult>`, and never returns
/// VK_INCOMPLETE.
///
/// The macro can take further arguments, which will be passed to the
/// API call.
///
/// By prepending `@void`, this macro can also be used on commands like
/// `vkGetPhysicalDeviceProperties`, which return `void` instead of
/// `VkResult`. In this case, it just returns `Vec<_>`.
///
/// # Examples
///
/// ```
/// vk_enumerate!(enumerate_physical_devices, instance).check()?;
/// vk_enumerate!(
///     @void get_physical_device_queue_family_properties,
///     physical_device,
/// ).check()?;
/// ```
#[macro_export]
macro_rules! enumerate {
    ($command:expr $(, $param:expr)*) => {
        $crate::enumerate_impl!(($command) ($($param,)*))
    };
    ($command:expr $(, $param:expr)*,) => {
        $crate::enumerate_impl!(($command) ($($param,)*))
    };
    (@void $command:expr $(, $param:expr)*) => {
        $crate::enumerate_impl!(@void ($command) ($($param,)*))
    };
    (@void $command:expr $(, $param:expr)*,) => {
        $crate::enumerate_impl!(@void ($command) ($($param,)*))
    };
}

/// Similar to `vk_enumerate`, except the new second argument is now
/// treated as a method of the first.
///
/// # Examples
///
/// ```
/// vk_enumerate2!(instance_wrapper, enumerate_physical_devices).check()?;
/// vk_enumerate2!(
///     @void instance_wrapper,
///     get_physical_device_queue_family_properties,
///     physical_device,
/// ).check()?;
/// ```
#[macro_export]
macro_rules! enumerate2 {
    ($object:expr, $method:ident $(, $param:expr)*) => {
        $crate::enumerate_impl!(($object.$method) ($($param,)*))
    };
    ($object:expr, $method:ident $(, $param:expr)*,) => {
        $crate::enumerate_impl!(($object.$method) ($($param,)*))
    };
    (@void $object:expr, $method:ident $(, $param:expr)*,) => {
        $crate::enumerate_impl!(@void ($object.$method) ($($param,)*))
    };
    (@void $object:expr, $method:ident $(, $param:expr)*,) => {
        $crate::enumerate_impl!(@void ($object.$method) ($($param,)*))
    };
}

/// A private macro used to implement `enumerate!`.
#[macro_export]
macro_rules! enumerate_impl {
    (($($command:tt)*) ($($param:expr,)*)) => {{
        let x: std::result::Result<_, $crate::Result> = try {
            let mut n: u32 = 0;
            $($command)*($($param,)* &mut n as *mut _, std::ptr::null_mut())
                .check()?;
            let mut v = std::vec::Vec::with_capacity(n as usize);
            $($command)*($($param,)* &mut n as *mut _, v.as_mut_ptr())
                .check()?;
            v.set_len(n as usize);
            v
        };
        x
    }};
    (@void ($($command:tt)*) ($($param:expr,)*)) => {{
        let mut n: u32 = 0;
        $($command)*($($param,)* &mut n as *mut _, std::ptr::null_mut());
        let mut v = std::vec::Vec::with_capacity(n as usize);
        $($command)*($($param,)* &mut n as *mut _, v.as_mut_ptr());
        v.set_len(n as usize);
        v
    }};
}
