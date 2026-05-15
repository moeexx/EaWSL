use super::storage::DiskSize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Install-time options accepted by `wsl.exe --install`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InstallOptions {
    pub name: Option<String>,
    pub location: Option<PathBuf>,
    pub vhd_size: Option<DiskSize>,
    pub fixed_vhd: bool,
}
