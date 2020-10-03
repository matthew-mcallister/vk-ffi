use crate as vk;

#[test]
fn struct_equality_with_padding() {
    let a = vk::PipelineColorBlendStateCreateInfo {
        attachment_count: 1,
        ..Default::default()
    };
    let mut b = vk::PipelineColorBlendStateCreateInfo {
        attachment_count: 1,
        ..Default::default()
    };
    assert_eq!(a, b);

    b.blend_constants = [1.0, 0.0, 0.0, 0.0];
    assert_ne!(a, b);
}

#[test]
fn struct_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let entry = vk::SpecializationMapEntry {
        constant_id: 0,
        offset: 0,
        size: 4,
    };
    set.insert(entry);
    set.insert(entry);
    assert_eq!(set.len(), 1);
    set.insert(vk::SpecializationMapEntry {
        constant_id: 1,
        offset: 4,
        size: 4,
    });
    assert_eq!(set.len(), 2);
}

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
