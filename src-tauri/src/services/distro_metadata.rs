use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::services::system::filesystem::{self, WriteTextError};

const DISTRIBUTION_INFO_URL: &str =
    "https://raw.githubusercontent.com/microsoft/WSL/master/distributions/DistributionInfo.json";
const RAW_FILE_NAME: &str = "distribution-info.json";
const CACHE_FILE_NAME: &str = "distro-metadata-cache.json";
const REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DistroMetadata {
    pub name: String,
    pub friendly_name: String,
    pub amd64_url: Option<String>,
    pub arm64_url: Option<String>,
    pub modern: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct DistroMetadataCache {
    source_url: String,
    last_success_at_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DistributionInfoFile {
    #[serde(default)]
    modern_distributions: HashMap<String, Vec<ModernDistribution>>,
    #[serde(default)]
    distributions: Vec<LegacyDistribution>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ModernDistribution {
    name: String,
    friendly_name: String,
    #[serde(rename = "Amd64Url")]
    amd64_url: Option<ArchitectureDownload>,
    #[serde(rename = "Arm64Url")]
    arm64_url: Option<ArchitectureDownload>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ArchitectureDownload {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LegacyDistribution {
    name: String,
    friendly_name: String,
    #[serde(rename = "Amd64PackageUrl")]
    amd64_package_url: Option<String>,
    #[serde(rename = "Arm64PackageUrl")]
    arm64_package_url: Option<String>,
}

pub async fn get(app: AppHandle) -> Result<Vec<DistroMetadata>, String> {
    let paths = DistroMetadataPaths::new(&app)?;
    tokio::task::spawn_blocking(move || read_cached_metadata(&paths.raw))
        .await
        .map_err(|error| format!("failed to join distro metadata read task: {error}"))
}

pub async fn refresh(app: AppHandle) -> Result<Vec<DistroMetadata>, String> {
    let paths = DistroMetadataPaths::new(&app)?;
    let now_ms = current_unix_time_ms();

    tokio::task::spawn_blocking(move || refresh_metadata_blocking(paths, now_ms))
        .await
        .map_err(|error| format!("failed to join distro metadata refresh task: {error}"))
}

fn refresh_metadata_blocking(paths: DistroMetadataPaths, now_ms: u64) -> Vec<DistroMetadata> {
    if is_cache_fresh(&paths.cache, now_ms) {
        return read_cached_metadata(&paths.raw);
    }

    match reqwest::blocking::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .and_then(|client| client.get(DISTRIBUTION_INFO_URL).send())
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())
    {
        Ok(raw) => {
            let metadata = parse_distribution_info(&raw).unwrap_or_default();
            if !metadata.is_empty() && write_cache_files(&paths, &raw, now_ms).is_ok() {
                return metadata;
            }
            read_cached_metadata(&paths.raw)
        }
        Err(_) => read_cached_metadata(&paths.raw),
    }
}

fn read_cached_metadata(raw_path: &Path) -> Vec<DistroMetadata> {
    let Ok(raw) = fs::read_to_string(raw_path) else {
        return Vec::new();
    };

    parse_distribution_info(&raw).unwrap_or_default()
}

fn parse_distribution_info(raw: &str) -> Result<Vec<DistroMetadata>, serde_json::Error> {
    let parsed = serde_json::from_str::<DistributionInfoFile>(raw)?;
    let mut modern_names = HashSet::new();
    let mut entries = Vec::new();

    for distribution in parsed.modern_distributions.into_values().flatten() {
        let entry = map_modern_distribution(distribution);
        modern_names.insert(normalize_distro_name_key(&entry.name));
        entries.push(entry);
    }

    entries.extend(
        parsed
            .distributions
            .into_iter()
            .map(map_legacy_distribution)
            .filter(|entry| !modern_names.contains(&normalize_distro_name_key(&entry.name))),
    );

    Ok(entries)
}

fn map_modern_distribution(value: ModernDistribution) -> DistroMetadata {
    DistroMetadata {
        name: value.name,
        friendly_name: value.friendly_name,
        amd64_url: value
            .amd64_url
            .as_ref()
            .and_then(|download| download.url.as_deref().and_then(normalize_optional_string)),
        arm64_url: value
            .arm64_url
            .as_ref()
            .and_then(|download| download.url.as_deref().and_then(normalize_optional_string)),
        modern: true,
    }
}

fn map_legacy_distribution(value: LegacyDistribution) -> DistroMetadata {
    DistroMetadata {
        name: value.name,
        friendly_name: value.friendly_name,
        amd64_url: value
            .amd64_package_url
            .as_deref()
            .and_then(normalize_optional_string),
        arm64_url: value
            .arm64_package_url
            .as_deref()
            .and_then(normalize_optional_string),
        modern: false,
    }
}

fn normalize_distro_name_key(value: &str) -> String {
    value.to_ascii_lowercase()
}

fn normalize_optional_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn is_cache_fresh(cache_path: &Path, now_ms: u64) -> bool {
    let Ok(raw) = fs::read_to_string(cache_path) else {
        return false;
    };
    let Ok(cache) = serde_json::from_str::<DistroMetadataCache>(&raw) else {
        return false;
    };

    cache.source_url == DISTRIBUTION_INFO_URL
        && now_ms.saturating_sub(cache.last_success_at_ms) < REFRESH_INTERVAL.as_millis() as u64
}

fn write_cache_files(paths: &DistroMetadataPaths, raw: &str, now_ms: u64) -> Result<(), String> {
    filesystem::write_text_creating_parent(&paths.raw, raw).map_err(format_write_error)?;

    let cache = DistroMetadataCache {
        source_url: DISTRIBUTION_INFO_URL.to_string(),
        last_success_at_ms: now_ms,
    };
    let raw_cache = serde_json::to_string_pretty(&cache)
        .map_err(|error| format!("failed to serialize distro metadata cache: {error}"))?;
    filesystem::write_text_creating_parent(&paths.cache, &raw_cache).map_err(format_write_error)
}

fn format_write_error(error: WriteTextError) -> String {
    match error {
        WriteTextError::CreateDirectory { path, source } => {
            format!(
                "failed to create distro metadata directory {}: {source}",
                path.display()
            )
        }
        WriteTextError::Write { path, source } => {
            format!(
                "failed to write distro metadata file {}: {source}",
                path.display()
            )
        }
    }
}

fn current_unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug)]
struct DistroMetadataPaths {
    raw: PathBuf,
    cache: PathBuf,
}

impl DistroMetadataPaths {
    fn new(app: &AppHandle) -> Result<Self, String> {
        let root = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("failed to resolve app data directory: {error}"))?;

        Ok(Self {
            raw: root.join(RAW_FILE_NAME),
            cache: root.join(CACHE_FILE_NAME),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{
        is_cache_fresh, parse_distribution_info, read_cached_metadata, write_cache_files,
        DistroMetadataCache, DistroMetadataPaths, DISTRIBUTION_INFO_URL, REFRESH_INTERVAL,
    };
    use crate::commands::test_support::unique_temp_path;

    fn unique_test_path(name: &str) -> PathBuf {
        unique_temp_path("distro-metadata", "test").join(name)
    }

    fn sample_distribution_info() -> &'static str {
        r#"{
  "Default": "Ubuntu",
  "Ignored": "ignored",
  "ModernDistributions": {
    "Ubuntu": [
      {
        "Name": "Ubuntu",
        "FriendlyName": "Ubuntu",
        "Amd64Url": {
          "Url": "https://example.test/ubuntu.appx",
          "Sha256": "abc123"
        },
        "Arm64Url": {
          "Url": "https://example.test/ubuntu-arm64.appx",
          "Sha256": "def456"
        },
        "StoreAppId": "ignored",
        "PackageFamilyName": "ignored"
      }
    ],
    "Other": [
      {
        "Name": "NoAmd64",
        "FriendlyName": "No Amd64"
      }
    ]
  },
  "Distributions": [
    {
      "Name": "ubuntu",
      "FriendlyName": "Legacy Ubuntu",
      "Amd64PackageUrl": "https://example.test/legacy-ubuntu.appx"
    },
    {
      "Name": "Debian",
      "FriendlyName": "Debian GNU/Linux",
      "Amd64PackageUrl": "https://example.test/debian.appx",
      "Arm64PackageUrl": "https://example.test/debian-arm64.appx"
    }
  ]
}"#
    }

    #[test]
    fn parse_distribution_info_maps_modern_and_legacy_entries() {
        let metadata =
            parse_distribution_info(sample_distribution_info()).expect("json should parse");
        let ubuntu = metadata
            .iter()
            .find(|entry| entry.name == "Ubuntu")
            .expect("modern Ubuntu should be returned");
        let no_amd64 = metadata
            .iter()
            .find(|entry| entry.name == "NoAmd64")
            .expect("modern entry without URLs should be returned");
        let debian = metadata
            .iter()
            .find(|entry| entry.name == "Debian")
            .expect("legacy Debian should be returned");

        assert_eq!(metadata.len(), 3);
        assert!(ubuntu.modern);
        assert_eq!(
            ubuntu.amd64_url.as_deref(),
            Some("https://example.test/ubuntu.appx")
        );
        assert_eq!(
            ubuntu.arm64_url.as_deref(),
            Some("https://example.test/ubuntu-arm64.appx")
        );
        assert_eq!(no_amd64.amd64_url, None);
        assert_eq!(no_amd64.arm64_url, None);
        assert_eq!(
            debian.amd64_url.as_deref(),
            Some("https://example.test/debian.appx")
        );
        assert_eq!(
            debian.arm64_url.as_deref(),
            Some("https://example.test/debian-arm64.appx")
        );
        assert!(!debian.modern);
        assert!(!metadata
            .iter()
            .any(|entry| entry.friendly_name == "Legacy Ubuntu"));
    }

    #[test]
    fn read_cached_metadata_returns_empty_for_missing_or_damaged_file() {
        let missing = unique_test_path("missing.json");
        assert!(read_cached_metadata(&missing).is_empty());

        let damaged = unique_test_path("damaged.json");
        fs::create_dir_all(damaged.parent().expect("test path should have parent"))
            .expect("test directory should be created");
        fs::write(&damaged, "{ not json").expect("damaged file should be written");

        assert!(read_cached_metadata(&damaged).is_empty());

        fs::remove_dir_all(damaged.parent().expect("test path should have parent"))
            .expect("test directory should be removed");
    }

    #[test]
    fn write_cache_files_persists_raw_and_manifest() {
        let root = unique_test_path("cache");
        let paths = DistroMetadataPaths {
            raw: root.join("distribution-info.json"),
            cache: root.join("distro-metadata-cache.json"),
        };

        write_cache_files(&paths, sample_distribution_info(), 123)
            .expect("cache files should be written");

        assert_eq!(
            fs::read_to_string(&paths.raw).expect("raw should be readable"),
            sample_distribution_info()
        );

        let cache = serde_json::from_str::<DistroMetadataCache>(
            &fs::read_to_string(&paths.cache).expect("cache should be readable"),
        )
        .expect("cache should parse");
        assert_eq!(cache.source_url, DISTRIBUTION_INFO_URL);
        assert_eq!(cache.last_success_at_ms, 123);

        fs::remove_dir_all(&root).expect("test directory should be removed");
    }

    #[test]
    fn is_cache_fresh_uses_24_hour_interval() {
        let path = unique_test_path("fresh-cache.json");
        fs::create_dir_all(path.parent().expect("test path should have parent"))
            .expect("test directory should be created");
        fs::write(
            &path,
            serde_json::to_string(&DistroMetadataCache {
                source_url: DISTRIBUTION_INFO_URL.to_string(),
                last_success_at_ms: 1_000,
            })
            .expect("cache should serialize"),
        )
        .expect("cache should be written");

        assert!(is_cache_fresh(
            &path,
            1_000 + REFRESH_INTERVAL.as_millis() as u64 - 1
        ));
        assert!(!is_cache_fresh(
            &path,
            1_000 + REFRESH_INTERVAL.as_millis() as u64
        ));

        fs::remove_dir_all(path.parent().expect("test path should have parent"))
            .expect("test directory should be removed");
    }
}
