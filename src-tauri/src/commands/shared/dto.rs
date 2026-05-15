use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use wsl_core::{DiskSize, ExportFormat, InstallOptions, WslCommandContext, WslError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstallDistroRequest {
    pub request_id: String,
    pub distro: String,
    pub options: InstallOptionsPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstallOptionsPayload {
    pub name: Option<String>,
    pub location: Option<PathBuf>,
    pub vhd_size: Option<String>,
    pub fixed_vhd: bool,
}

impl InstallOptionsPayload {
    pub(crate) fn into_core(self) -> Result<InstallOptions, WslError> {
        let vhd_size = self
            .vhd_size
            .as_deref()
            .map(DiskSize::parse)
            .transpose()
            .map_err(|err| WslError::InvalidArgument {
                context: WslCommandContext::Install,
                raw_output: err.to_string(),
            })?;

        Ok(InstallOptions {
            name: self.name,
            location: self.location,
            vhd_size,
            fixed_vhd: self.fixed_vhd,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportDistroRequest {
    pub request_id: String,
    pub distro: String,
    pub location: PathBuf,
    pub file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportDistroInPlaceRequest {
    pub request_id: String,
    pub distro: String,
    pub source_vhdx: PathBuf,
    pub target_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportDistroRequest {
    pub request_id: String,
    pub distro: String,
    pub file: PathBuf,
    pub format: ExportFormat,
}

#[cfg(test)]
mod tests {
    use super::{
        ExportDistroRequest, ImportDistroInPlaceRequest, InstallDistroRequest,
        InstallOptionsPayload,
    };
    use wsl_core::ExportFormat;
    use wsl_core::{WslCommandContext, WslError};

    #[test]
    fn install_options_payload_maps_to_core_options() {
        let payload = InstallOptionsPayload {
            name: Some("custom-ubuntu".to_string()),
            location: Some("D:/WSL/Ubuntu".into()),
            vhd_size: Some("1.235 tb".to_string()),
            fixed_vhd: true,
        };

        let options = payload
            .into_core()
            .expect("payload should map to core options");

        assert_eq!(options.name.as_deref(), Some("custom-ubuntu"));
        assert_eq!(
            options.location.as_deref(),
            Some(std::path::Path::new("D:/WSL/Ubuntu"))
        );
        assert_eq!(
            options.vhd_size.as_ref().map(|size| size.as_str()),
            Some("1.24TB")
        );
        assert!(options.fixed_vhd);
    }

    #[test]
    fn install_options_payload_rejects_invalid_disk_size() {
        let payload = InstallOptionsPayload {
            name: None,
            location: None,
            vhd_size: Some("20G".to_string()),
            fixed_vhd: false,
        };

        let err = payload
            .into_core()
            .expect_err("invalid disk size should fail");

        assert!(matches!(
            err,
            WslError::InvalidArgument {
                context: WslCommandContext::Install,
                ..
            }
        ));
    }

    #[test]
    fn install_request_deserializes_existing_wire_shape() {
        let request = serde_json::from_str::<InstallDistroRequest>(
            r#"{
              "requestId": "req-install",
              "distro": "Ubuntu",
              "options": {
                "name": null,
                "location": "D:/WSL/Ubuntu",
                "vhd_size": "20GB",
                "fixed_vhd": false
              }
            }"#,
        )
        .expect("existing install request wire shape should deserialize");

        assert_eq!(request.request_id, "req-install");
        assert_eq!(request.options.vhd_size.as_deref(), Some("20GB"));
    }

    #[test]
    fn import_in_place_request_deserializes_camel_case_wire_shape() {
        let request = serde_json::from_str::<ImportDistroInPlaceRequest>(
            r#"{
              "requestId": "req-vhdx",
              "distro": "Ubuntu",
              "sourceVhdx": "D:/images/ext4.vhdx",
              "targetDirectory": "D:/WSL/Ubuntu"
            }"#,
        )
        .expect("import-in-place request wire shape should deserialize");

        assert_eq!(request.request_id, "req-vhdx");
        assert_eq!(
            request.source_vhdx.as_path(),
            std::path::Path::new("D:/images/ext4.vhdx")
        );
        assert_eq!(
            request.target_directory.as_deref(),
            Some(std::path::Path::new("D:/WSL/Ubuntu"))
        );
    }

    #[test]
    fn export_request_deserializes_camel_case_wire_shape() {
        let request = serde_json::from_str::<ExportDistroRequest>(
            r#"{
              "requestId": "req-export",
              "distro": "Ubuntu",
              "file": "D:/exports/ubuntu.tar.gz",
              "format": "TarGz"
            }"#,
        )
        .expect("export request wire shape should deserialize");

        assert_eq!(request.request_id, "req-export");
        assert_eq!(request.distro, "Ubuntu");
        assert_eq!(
            request.file.as_path(),
            std::path::Path::new("D:/exports/ubuntu.tar.gz")
        );
        assert_eq!(request.format, ExportFormat::TarGz);
    }
}
