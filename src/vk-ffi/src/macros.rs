#[macro_export]
macro_rules! vk_make_version {
        ($major:expr, $minor:expr, $patch:expr) => {
        ($major << 22) | ($minor << 12) | $patch
    }
}

#[macro_export]
macro_rules! vk_version_major {
    ($version:expr) => { version >> 22 }
}

#[macro_export]
macro_rules! vk_version_minor {
    ($version:expr) => { (version >> 12) & 0x3ff }
}

#[macro_export]
macro_rules! vk_version_patch {
    ($version:expr) => { version & 0xfff }
}

/// Converts a `VkResult` to a `Result<VkResult, VkResult>`, branching
/// on whether the code signifies an error.
#[macro_export]
macro_rules! vk_check {
    ($res:expr) => { if $res.is_success() { Ok($res) } else { Err($res) } }
}

/// Handles the boilerplate of making two calls to `VkEnumerate*`: one
/// to get the number of elements, and another to fill the array. This
/// macro simply fills a `Vec` and yields `Result<Vec, VkResult>`.
#[macro_export]
macro_rules! vk_enumerate {
    ($command:expr) => {{
        let res: ::std::result::Result<::std::vec::Vec<_>, $crate::Result> =
            try
        {
            let mut num_elems: u32 = 0;
            $command(
                &mut num_elems as *mut _,
                ::std::ptr::null_mut(),
            ).check()?;
            let mut vec = ::std::vec::Vec::with_capacity(num_elems as usize);
            $command(&mut num_elems as *mut _, vec.as_mut_ptr()).check()?;
            vec.set_len(num_elems as usize);
            vec
        };
        res
    }};
    ($command:expr, $object:expr) => {{
        let res: ::std::result::Result<::std::vec::Vec<_>, $crate::Result> =
            try
        {
            let mut num_elems: u32 = 0;
            $command(
                $object,
                &mut num_elems as *mut _,
                ::std::ptr::null_mut(),
            ).check()?;
            let mut vec = ::std::vec::Vec::with_capacity(num_elems as usize);
            $command($object, &mut num_elems as *mut _, vec.as_mut_ptr())
                .check()?;
            vec.set_len(num_elems as usize);
            vec
        };
        res
    }};
}
