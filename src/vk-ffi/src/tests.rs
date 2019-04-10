#[cfg(feature = "reflection")]
mod reflection {
    use std::str::FromStr;

    use crate as vk;
    use crate::reflection::{AggregateInfo, EnumInfo, ParseEnumError};

    #[test]
    fn test_enum_from_str() -> Result<(), ParseEnumError> {
        assert_eq!(vk::Format::from_str("R8_UNORM")?, vk::Format::R8_UNORM);
        assert_eq!(
            vk::CullModeFlags::from_str("FRONT_AND_BACK")?,
            vk::CullModeFlags::FRONT_AND_BACK,
        );
        Ok(())
    }

    #[test]
    fn test_enum_from_str_failure() {
        assert!(vk::Format::from_str("WAT").is_err());
        assert!(vk::CullModeFlags::from_str("LOL").is_err());
    }

    #[test]
    fn test_aggregate_fields() {
        assert_eq!(
            vk::Viewport::FIELDS,
            &["x", "y", "width", "height", "min_depth", "max_depth"],
        );
    }

    #[test]
    fn test_enum_members() {
        assert_eq!(
            vk::CullModeFlags::MEMBERS,
            &["NONE", "FRONT_BIT", "BACK_BIT", "FRONT_AND_BACK"],
        );
        assert_eq!(
            vk::CullModeFlags::VALUES,
            &[
                vk::CullModeFlags::NONE,
                vk::CullModeFlags::FRONT_BIT,
                vk::CullModeFlags::BACK_BIT,
                vk::CullModeFlags::FRONT_AND_BACK,
            ],
        );
    }
}
