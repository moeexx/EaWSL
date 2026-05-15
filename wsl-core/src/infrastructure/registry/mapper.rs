use std::path::PathBuf;

use crate::domain::model::distro::RegisteredDistroMetadata;

use super::reader::RegistryRawRecord;

const VERBATIM_PATH_PREFIX: &str = "\\\\?\\";

pub(super) fn map_registry_record(raw: RegistryRawRecord) -> RegisteredDistroMetadata {
    RegisteredDistroMetadata {
        name: raw.distribution_name,
        version: raw.version,
        base_path: raw
            .base_path
            .and_then(normalize_optional_string)
            .map(|value| PathBuf::from(strip_verbatim_prefix(&value))),
        vhd_file_name: raw.vhd_file_name.and_then(normalize_optional_string),
        flavor: raw.flavor.and_then(normalize_optional_string),
        os_version: raw.os_version.and_then(normalize_optional_string),
        default_uid: raw.default_uid,
    }
}

pub(super) fn normalize_optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn strip_verbatim_prefix(value: &str) -> String {
    value
        .strip_prefix(VERBATIM_PATH_PREFIX)
        .unwrap_or(value)
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::map_registry_record;
    use crate::infrastructure::registry::reader::RegistryRawRecord;

    #[test]
    fn map_registry_record_normalizes_optional_strings_and_base_path() {
        let mapped = map_registry_record(RegistryRawRecord {
            distribution_name: "docker-desktop".to_string(),
            version: 2,
            base_path: Some(r"\\?\D:\WSL\Ubuntu".to_string()),
            vhd_file_name: Some("ext4.vhdx".to_string()),
            flavor: Some("   ".to_string()),
            os_version: Some("".to_string()),
            default_uid: Some(1000),
        });

        assert_eq!(mapped.name, "docker-desktop");
        assert_eq!(mapped.base_path, Some(PathBuf::from(r"D:\WSL\Ubuntu")));
        assert_eq!(mapped.vhd_file_name.as_deref(), Some("ext4.vhdx"));
        assert_eq!(mapped.flavor, None);
        assert_eq!(mapped.os_version, None);
        assert_eq!(mapped.default_uid, Some(1000));
    }
}
