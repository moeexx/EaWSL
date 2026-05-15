use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Parsed distro state from `wsl.exe --list --verbose`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DistroState {
    Running,
    Stopped,
    Installing,
    Unknown(String),
}

/// Merged distro info from `wsl.exe` output and registry metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistroInfo {
    pub name: String,
    pub state: DistroState,
    pub version: u8,
    pub is_default: bool,
    pub base_path: Option<PathBuf>,
    pub vhd_file_name: Option<String>,
    pub flavor: Option<String>,
    pub os_version: Option<String>,
    pub default_uid: Option<u32>,
}

/// Structured output from `wsl.exe --version`; `wsl` and `windows` stay required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WslVersion {
    pub wsl: String,
    pub kernel: Option<String>,
    pub wslg: Option<String>,
    pub msrdc: Option<String>,
    pub direct3d: Option<String>,
    pub dxcore: Option<String>,
    pub windows: String,
}

/// One installable distro entry from `wsl.exe --list --online`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnlineDistro {
    pub name: String,
    pub friendly_name: String,
}

/// Internal snapshot before registry metadata is merged.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InstalledDistroSnapshot {
    pub name: String,
    pub state: DistroState,
    pub version: u8,
    pub is_default: bool,
}

/// Registry-side metadata used by query orchestration.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RegisteredDistroMetadata {
    pub name: String,
    pub version: u8,
    pub base_path: Option<PathBuf>,
    pub vhd_file_name: Option<String>,
    pub flavor: Option<String>,
    pub os_version: Option<String>,
    pub default_uid: Option<u32>,
}
