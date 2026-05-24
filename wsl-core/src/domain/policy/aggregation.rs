use std::collections::HashMap;

use crate::domain::model::distro::{InstalledDistroSnapshot, RegisteredDistroMetadata};
use crate::{DistroInfo, DistroState};

pub(crate) fn aggregate_distros(
    cli_entries: Vec<InstalledDistroSnapshot>,
    registry_entries: Vec<RegisteredDistroMetadata>,
) -> Vec<DistroInfo> {
    let mut registry_by_name = registry_entries
        .into_iter()
        .map(|entry| (normalized_distro_name_key(&entry.name), entry))
        .collect::<HashMap<_, _>>();

    let mut distros = cli_entries
        .into_iter()
        .map(|cli_entry| {
            let registry_entry =
                registry_by_name.remove(&normalized_distro_name_key(&cli_entry.name));
            merge_cli_and_registry(cli_entry, registry_entry)
        })
        .collect::<Vec<_>>();

    let mut registry_only = registry_by_name.into_values().collect::<Vec<_>>();
    registry_only.sort_by(|left, right| left.name.cmp(&right.name));
    distros.extend(registry_only.into_iter().map(registry_only_distro));
    distros
}

fn normalized_distro_name_key(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn merge_cli_and_registry(
    cli_entry: InstalledDistroSnapshot,
    registry_entry: Option<RegisteredDistroMetadata>,
) -> DistroInfo {
    let metadata = registry_entry.as_ref();

    DistroInfo {
        name: cli_entry.name,
        state: cli_entry.state,
        version: cli_entry.version,
        is_default: cli_entry.is_default,
        base_path: metadata.and_then(|entry| entry.base_path.clone()),
        vhd_file_name: metadata.and_then(|entry| entry.vhd_file_name.clone()),
        flavor: metadata.and_then(|entry| entry.flavor.clone()),
        os_version: metadata.and_then(|entry| entry.os_version.clone()),
        default_uid: metadata.and_then(|entry| entry.default_uid),
    }
}

fn registry_only_distro(entry: RegisteredDistroMetadata) -> DistroInfo {
    DistroInfo {
        name: entry.name,
        state: DistroState::Installing,
        version: entry.version,
        is_default: false,
        base_path: entry.base_path,
        vhd_file_name: entry.vhd_file_name,
        flavor: entry.flavor,
        os_version: entry.os_version,
        default_uid: entry.default_uid,
    }
}

#[cfg(test)]
mod tests {
    use super::aggregate_distros;
    use crate::domain::model::distro::{InstalledDistroSnapshot, RegisteredDistroMetadata};
    use crate::{DistroInfo, DistroState};

    #[test]
    fn aggregate_distros_merges_cli_and_registry_metadata() {
        let distros = aggregate_distros(
            vec![InstalledDistroSnapshot {
                name: "Ubuntu".to_string(),
                state: DistroState::Running,
                version: 2,
                is_default: true,
            }],
            vec![RegisteredDistroMetadata {
                name: "Ubuntu".to_string(),
                version: 2,
                base_path: Some("D:/WSL/Ubuntu".into()),
                vhd_file_name: Some("ext4.vhdx".to_string()),
                flavor: Some("ubuntu".to_string()),
                os_version: Some("24.04".to_string()),
                default_uid: Some(1000),
            }],
        );

        assert_eq!(
            distros,
            vec![DistroInfo {
                name: "Ubuntu".to_string(),
                state: DistroState::Running,
                version: 2,
                is_default: true,
                base_path: Some("D:/WSL/Ubuntu".into()),
                vhd_file_name: Some("ext4.vhdx".to_string()),
                flavor: Some("ubuntu".to_string()),
                os_version: Some("24.04".to_string()),
                default_uid: Some(1000),
            }]
        );
    }

    #[test]
    fn aggregate_distros_keeps_cli_only_entries_and_appends_registry_only_as_installing() {
        let distros = aggregate_distros(
            vec![InstalledDistroSnapshot {
                name: "Ubuntu".to_string(),
                state: DistroState::Stopped,
                version: 2,
                is_default: false,
            }],
            vec![RegisteredDistroMetadata {
                name: "docker-desktop".to_string(),
                version: 2,
                base_path: Some("D:/WSL/docker-desktop".into()),
                vhd_file_name: None,
                flavor: None,
                os_version: None,
                default_uid: None,
            }],
        );

        assert_eq!(
            distros,
            vec![
                DistroInfo {
                    name: "Ubuntu".to_string(),
                    state: DistroState::Stopped,
                    version: 2,
                    is_default: false,
                    base_path: None,
                    vhd_file_name: None,
                    flavor: None,
                    os_version: None,
                    default_uid: None,
                },
                DistroInfo {
                    name: "docker-desktop".to_string(),
                    state: DistroState::Installing,
                    version: 2,
                    is_default: false,
                    base_path: Some("D:/WSL/docker-desktop".into()),
                    vhd_file_name: None,
                    flavor: None,
                    os_version: None,
                    default_uid: None,
                },
            ]
        );
    }

    #[test]
    fn aggregate_distros_matches_names_case_insensitively_and_keeps_cli_casing() {
        let distros = aggregate_distros(
            vec![InstalledDistroSnapshot {
                name: "Debian".to_string(),
                state: DistroState::Running,
                version: 2,
                is_default: false,
            }],
            vec![RegisteredDistroMetadata {
                name: "debian".to_string(),
                version: 2,
                base_path: Some("D:/WSL/debian".into()),
                vhd_file_name: Some("ext4.vhdx".to_string()),
                flavor: Some("debian".to_string()),
                os_version: Some("12".to_string()),
                default_uid: Some(1000),
            }],
        );

        assert_eq!(
            distros,
            vec![DistroInfo {
                name: "Debian".to_string(),
                state: DistroState::Running,
                version: 2,
                is_default: false,
                base_path: Some("D:/WSL/debian".into()),
                vhd_file_name: Some("ext4.vhdx".to_string()),
                flavor: Some("debian".to_string()),
                os_version: Some("12".to_string()),
                default_uid: Some(1000),
            }]
        );
    }
}
