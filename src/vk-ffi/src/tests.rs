#[cfg(feature = "extra-traits")]
mod extra_traits {
    use std::str::FromStr;

    use crate as vk;
    use crate::internal::ParseEnumError;

    #[test]
    fn test_from_str() -> Result<(), ParseEnumError> {
        assert_eq!(vk::Format::from_str("R8_UNORM")?, vk::Format::R8_UNORM);
        assert_eq!(
            vk::CullModeFlags::from_str("FRONT_AND_BACK")?,
            vk::CullModeFlags::FRONT_AND_BACK,
        );
        Ok(())
    }

    #[test]
    fn test_from_str_failure() {
        assert!(vk::Format::from_str("WAT").is_err());
        assert!(vk::CullModeFlags::from_str("LOL").is_err());
    }
}
