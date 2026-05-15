use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::services::system::filesystem::{self, WriteTextError};

const DEFAULT_INSTALL_DIR_NAME: &str = "wsl";
const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_BACKGROUND_REFRESH_INTERVAL_MINUTES: u16 = 15;
const MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES: u16 = 1;
const MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES: u16 = 1440;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_install_location: String,
    pub background_refresh: BackgroundRefreshSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundRefreshSettings {
    pub interval_minutes: u16,
    pub targets: Vec<BackgroundRefreshTarget>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BackgroundRefreshTarget {
    Distros,
    SystemOverviewStorage,
    WslVersion,
    OnlineDistros,
}

impl Default for BackgroundRefreshSettings {
    fn default() -> Self {
        Self {
            interval_minutes: DEFAULT_BACKGROUND_REFRESH_INTERVAL_MINUTES,
            targets: vec![
                BackgroundRefreshTarget::Distros,
                BackgroundRefreshTarget::SystemOverviewStorage,
                BackgroundRefreshTarget::WslVersion,
            ],
        }
    }
}

#[derive(Debug)]
enum SettingsWriteError {
    EmptyDefaultInstallLocation,
    InvalidBackgroundRefreshInterval { value: u16 },
    EmptyBackgroundRefreshTargets,
    CreateDirectory { path: PathBuf, source: io::Error },
    Serialize(serde_json::Error),
    Write { path: PathBuf, source: io::Error },
}

impl fmt::Display for SettingsWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDefaultInstallLocation => {
                write!(f, "Default install location cannot be empty.")
            }
            Self::InvalidBackgroundRefreshInterval { value } => {
                write!(
                    f,
                    "Background refresh interval must be between {MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES} and {MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES} minutes, got {value}."
                )
            }
            Self::EmptyBackgroundRefreshTargets => {
                write!(f, "Background refresh targets cannot be empty.")
            }
            Self::CreateDirectory { path, source } => {
                write!(
                    f,
                    "failed to create settings directory {}: {source}",
                    path.display()
                )
            }
            Self::Serialize(source) => write!(f, "failed to serialize settings: {source}"),
            Self::Write { path, source } => {
                write!(
                    f,
                    "failed to write settings file {}: {source}",
                    path.display()
                )
            }
        }
    }
}

pub async fn get(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_file_path(&app)?;
    let default_install_location = default_install_location(&app)?;
    read_app_settings(path, default_install_location).await
}

pub async fn save(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let path = settings_file_path(&app)?;
    write_app_settings(path, settings).await
}

fn settings_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join(SETTINGS_FILE_NAME))
        .map_err(|error| format!("failed to resolve app data directory: {error}"))
}

fn default_install_location(app: &AppHandle) -> Result<String, String> {
    app.path()
        .local_data_dir()
        .map(|dir| {
            dir.join(DEFAULT_INSTALL_DIR_NAME)
                .to_string_lossy()
                .into_owned()
        })
        .map_err(|error| format!("failed to resolve default install location: {error}"))
}

async fn read_app_settings(
    path: PathBuf,
    default_install_location: String,
) -> Result<AppSettings, String> {
    tokio::task::spawn_blocking(move || {
        read_app_settings_blocking(&path, &default_install_location)
    })
    .await
    .map_err(|error| format!("failed to join settings read task: {error}"))
}

async fn write_app_settings(path: PathBuf, settings: AppSettings) -> Result<AppSettings, String> {
    tokio::task::spawn_blocking(move || write_app_settings_blocking(&path, settings))
        .await
        .map_err(|error| format!("failed to join settings write task: {error}"))?
        .map_err(|error| error.to_string())
}

fn default_app_settings(default_install_location: &str) -> AppSettings {
    AppSettings {
        default_install_location: default_install_location.to_string(),
        background_refresh: BackgroundRefreshSettings::default(),
    }
}

fn read_app_settings_blocking(path: &Path, default_install_location: &str) -> AppSettings {
    let Ok(raw) = fs::read_to_string(path) else {
        return default_app_settings(default_install_location);
    };

    serde_json::from_str(&raw).unwrap_or_else(|_| default_app_settings(default_install_location))
}

fn write_app_settings_blocking(
    path: &Path,
    settings: AppSettings,
) -> Result<AppSettings, SettingsWriteError> {
    let trimmed_location = settings.default_install_location.trim();

    if trimmed_location.is_empty() {
        return Err(SettingsWriteError::EmptyDefaultInstallLocation);
    }

    let background_refresh = normalize_background_refresh(settings.background_refresh)?;

    let normalized_settings = AppSettings {
        default_install_location: trimmed_location.to_string(),
        background_refresh,
    };

    let raw = serde_json::to_string_pretty(&normalized_settings)
        .map_err(SettingsWriteError::Serialize)?;

    filesystem::write_text_creating_parent(path, &raw).map_err(SettingsWriteError::from)?;

    Ok(normalized_settings)
}

impl From<WriteTextError> for SettingsWriteError {
    fn from(error: WriteTextError) -> Self {
        match error {
            WriteTextError::CreateDirectory { path, source } => {
                Self::CreateDirectory { path, source }
            }
            WriteTextError::Write { path, source } => Self::Write { path, source },
        }
    }
}

fn normalize_background_refresh(
    settings: BackgroundRefreshSettings,
) -> Result<BackgroundRefreshSettings, SettingsWriteError> {
    if settings.interval_minutes < MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES
        || settings.interval_minutes > MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES
    {
        return Err(SettingsWriteError::InvalidBackgroundRefreshInterval {
            value: settings.interval_minutes,
        });
    }

    let targets = ordered_background_refresh_targets()
        .iter()
        .copied()
        .filter(|target| settings.targets.contains(target))
        .collect::<Vec<_>>();

    if targets.is_empty() {
        return Err(SettingsWriteError::EmptyBackgroundRefreshTargets);
    }

    Ok(BackgroundRefreshSettings {
        interval_minutes: settings.interval_minutes,
        targets,
    })
}

fn ordered_background_refresh_targets() -> &'static [BackgroundRefreshTarget] {
    &[
        BackgroundRefreshTarget::Distros,
        BackgroundRefreshTarget::SystemOverviewStorage,
        BackgroundRefreshTarget::WslVersion,
        BackgroundRefreshTarget::OnlineDistros,
    ]
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{
        default_app_settings, read_app_settings_blocking, write_app_settings_blocking, AppSettings,
        BackgroundRefreshSettings, BackgroundRefreshTarget,
    };
    use crate::commands::test_support::unique_temp_path;

    fn unique_test_path(name: &str) -> PathBuf {
        unique_temp_path("settings", "test").join(name)
    }

    #[test]
    fn write_settings_rejects_empty_location() {
        let path = unique_test_path("settings.json");
        let err = write_app_settings_blocking(
            &path,
            AppSettings {
                default_install_location: "   ".to_string(),
                background_refresh: BackgroundRefreshSettings::default(),
            },
        )
        .expect_err("empty location should fail");

        assert_eq!(err.to_string(), "Default install location cannot be empty.");
    }

    #[test]
    fn write_and_read_settings_round_trips_location() {
        let path = unique_test_path("settings.json");
        let saved = write_app_settings_blocking(
            &path,
            AppSettings {
                default_install_location: "  D:\\WSL  ".to_string(),
                background_refresh: BackgroundRefreshSettings {
                    interval_minutes: 5,
                    targets: vec![
                        BackgroundRefreshTarget::OnlineDistros,
                        BackgroundRefreshTarget::Distros,
                        BackgroundRefreshTarget::Distros,
                        BackgroundRefreshTarget::WslVersion,
                    ],
                },
            },
        )
        .expect("settings should be saved");

        assert_eq!(saved.default_install_location, r"D:\WSL");
        assert_eq!(
            saved.background_refresh.targets,
            vec![
                BackgroundRefreshTarget::Distros,
                BackgroundRefreshTarget::WslVersion,
                BackgroundRefreshTarget::OnlineDistros,
            ]
        );

        let read = read_app_settings_blocking(&path, "D:\\DefaultWSL");
        assert_eq!(read, saved);

        let parent = path.parent().expect("test path should have parent");
        fs::remove_dir_all(parent).expect("test directory should be removed");
    }

    #[test]
    fn read_settings_falls_back_for_missing_or_damaged_file() {
        let default_install_location = "D:\\DefaultWSL";
        let missing = unique_test_path("missing-settings.json");
        let missing_settings = read_app_settings_blocking(&missing, default_install_location);
        assert_eq!(
            missing_settings,
            default_app_settings(default_install_location)
        );

        let damaged = unique_test_path("damaged-settings.json");
        let damaged_parent = damaged.parent().expect("test path should have parent");
        fs::create_dir_all(damaged_parent).expect("test directory should be created");
        fs::write(&damaged, "{ not json").expect("damaged settings should be written");
        assert_eq!(
            read_app_settings_blocking(&damaged, default_install_location),
            default_app_settings(default_install_location)
        );
        fs::remove_dir_all(damaged_parent).expect("test directory should be removed");
    }

    #[test]
    fn write_settings_rejects_invalid_background_refresh() {
        let path = unique_test_path("settings.json");
        let interval_err = write_app_settings_blocking(
            &path,
            AppSettings {
                default_install_location: "D:\\WSL".to_string(),
                background_refresh: BackgroundRefreshSettings {
                    interval_minutes: 0,
                    targets: vec![BackgroundRefreshTarget::Distros],
                },
            },
        )
        .expect_err("zero interval should fail");
        assert_eq!(
            interval_err.to_string(),
            "Background refresh interval must be between 1 and 1440 minutes, got 0."
        );

        let interval_err = write_app_settings_blocking(
            &path,
            AppSettings {
                default_install_location: "D:\\WSL".to_string(),
                background_refresh: BackgroundRefreshSettings {
                    interval_minutes: 1441,
                    targets: vec![BackgroundRefreshTarget::Distros],
                },
            },
        )
        .expect_err("too large interval should fail");
        assert_eq!(
            interval_err.to_string(),
            "Background refresh interval must be between 1 and 1440 minutes, got 1441."
        );

        let targets_err = write_app_settings_blocking(
            &path,
            AppSettings {
                default_install_location: "D:\\WSL".to_string(),
                background_refresh: BackgroundRefreshSettings {
                    interval_minutes: 15,
                    targets: Vec::new(),
                },
            },
        )
        .expect_err("empty targets should fail");
        assert_eq!(
            targets_err.to_string(),
            "Background refresh targets cannot be empty."
        );
    }
}
