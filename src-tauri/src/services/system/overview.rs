use std::{process::Stdio, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
};

const SYSTEM_OVERVIEW_ERROR_MESSAGE: &str = "Failed to read system overview information.";
const HOST_COMMAND_TIMEOUT_MESSAGE: &str =
    "The host command timed out before a stable result was available.";
//TODO: Validate this timeout against collected host samples.
const HOST_QUERY_TIMEOUT_MS: u64 = 4_000;
const SYSTEM_OVERVIEW_CONTEXT: &str = "system_overview";
const SYSTEM_OVERVIEW_SCRIPT: &str = include_str!("overview.ps1");
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SystemOverviewScope {
    Full,
    Storage,
}

impl SystemOverviewScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Storage => "storage",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SystemOverview {
    pub windows: WindowsOverview,
    pub cpu: CpuOverview,
    pub memory: MemoryOverview,
    pub gpu: Option<GpuOverview>,
    pub storage: StorageOverview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowsOverview {
    pub product_name: Option<String>,
    pub display_version: Option<String>,
    pub build_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CpuOverview {
    pub model: Option<String>,
    pub max_clock_mhz: Option<u32>,
    pub core_count: Option<u32>,
    pub logical_processor_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MemoryOverview {
    pub total_bytes: Option<u64>,
    pub speed_mts: Option<u32>,
    pub used_slots: Option<u32>,
    pub total_slots: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GpuOverview {
    pub name: Option<String>,
    pub memory_bytes: Option<u64>,
    pub driver_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StorageOverview {
    pub total_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
    pub volume_count: Option<u32>,
}

#[derive(Debug)]
enum SystemOverviewQueryError {
    Spawn(std::io::Error),
    Io(std::io::Error),
    TimedOut { timeout_ms: u64 },
    CommandFailed { status: Option<i32>, stderr: String },
    Json(serde_json::Error),
}

impl SystemOverviewQueryError {
    fn to_user_message(&self) -> String {
        match self {
            Self::TimedOut { .. } => HOST_COMMAND_TIMEOUT_MESSAGE.to_string(),
            _ => SYSTEM_OVERVIEW_ERROR_MESSAGE.to_string(),
        }
    }
}

impl std::fmt::Display for SystemOverviewQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(err) => write!(f, "failed to spawn PowerShell: {err}"),
            Self::Io(err) => write!(f, "failed to read PowerShell output: {err}"),
            Self::TimedOut { timeout_ms } => write!(
                f,
                "host command timed out in {} after {} ms",
                SYSTEM_OVERVIEW_CONTEXT, timeout_ms
            ),
            Self::CommandFailed { status, stderr } => {
                write!(
                    f,
                    "PowerShell command failed with status {status:?}: {stderr}"
                )
            }
            Self::Json(err) => write!(f, "failed to parse system overview JSON: {err}"),
        }
    }
}

pub async fn get_system_overview(
    scope: Option<SystemOverviewScope>,
) -> Result<SystemOverview, String> {
    let scope = scope.unwrap_or(SystemOverviewScope::Full);

    read_system_overview(scope)
        .await
        .map_err(|err| err.to_user_message())
}

async fn read_system_overview(
    scope: SystemOverviewScope,
) -> Result<SystemOverview, SystemOverviewQueryError> {
    let stdout = run_system_overview_script(scope).await?;
    parse_system_overview(&stdout)
}

async fn run_system_overview_script(
    scope: SystemOverviewScope,
) -> Result<Vec<u8>, SystemOverviewQueryError> {
    let mut command = Command::new("powershell.exe");
    hide_child_console_window(&mut command);
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            SYSTEM_OVERVIEW_SCRIPT,
        ])
        .env("EAWSL_SYSTEM_OVERVIEW_SCOPE", scope.as_str())
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(SystemOverviewQueryError::Spawn)?;
    let mut stdout = child.stdout.take().ok_or_else(|| {
        SystemOverviewQueryError::Io(std::io::Error::other("missing stdout pipe"))
    })?;
    let mut stderr = child.stderr.take().ok_or_else(|| {
        SystemOverviewQueryError::Io(std::io::Error::other("missing stderr pipe"))
    })?;

    let timeout = Duration::from_millis(HOST_QUERY_TIMEOUT_MS);
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    let status = tokio::select! {
        output = async {
            tokio::try_join!(
                stdout.read_to_end(&mut stdout_bytes),
                stderr.read_to_end(&mut stderr_bytes),
                child.wait(),
            )
        } => {
            let (_, _, status) = output.map_err(SystemOverviewQueryError::Io)?;
            status
        },
        _ = tokio::time::sleep(timeout) => {
            cleanup_child(&mut child).await.map_err(SystemOverviewQueryError::Io)?;
            return Err(SystemOverviewQueryError::TimedOut {
                timeout_ms: HOST_QUERY_TIMEOUT_MS,
            });
        }
    };

    if !status.success() {
        return Err(SystemOverviewQueryError::CommandFailed {
            status: status.code(),
            stderr: String::from_utf8_lossy(&stderr_bytes).trim().to_string(),
        });
    }

    Ok(stdout_bytes)
}

#[cfg(windows)]
fn hide_child_console_window(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_child_console_window(_command: &mut Command) {}

async fn cleanup_child(child: &mut Child) -> std::io::Result<()> {
    match child.kill().await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => {}
        Err(error) => return Err(error),
    }

    match child.wait().await {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => Ok(()),
        Err(error) => Err(error),
    }
}

fn parse_system_overview(stdout: &[u8]) -> Result<SystemOverview, SystemOverviewQueryError> {
    let text = String::from_utf8_lossy(stdout);
    let payload = text.trim().trim_start_matches('\u{feff}');

    serde_json::from_str(payload).map_err(SystemOverviewQueryError::Json)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_system_overview, CpuOverview, GpuOverview, MemoryOverview, StorageOverview,
        SystemOverview, SystemOverviewQueryError, SystemOverviewScope, WindowsOverview,
        HOST_COMMAND_TIMEOUT_MESSAGE, HOST_QUERY_TIMEOUT_MS, SYSTEM_OVERVIEW_CONTEXT,
        SYSTEM_OVERVIEW_ERROR_MESSAGE,
    };

    #[test]
    fn parse_system_overview_deserializes_contract_shape() {
        let payload = br#"{
          "windows": {
            "productName": "Windows 11 Pro",
            "displayVersion": "24H2",
            "buildNumber": "26100.8246"
          },
          "cpu": {
            "model": "AMD Ryzen 9 7945HX with Radeon Graphics",
            "maxClockMhz": 2500,
            "coreCount": 16,
            "logicalProcessorCount": 32
          },
          "memory": {
            "totalBytes": 68719476736,
            "speedMts": 5200,
            "usedSlots": 2,
            "totalSlots": 2
          },
          "gpu": {
            "name": "AMD Radeon(TM) 610M",
            "memoryBytes": 4294967296,
            "driverVersion": "32.0.11018.1000"
          },
          "storage": {
            "totalBytes": 3000000000000,
            "usedBytes": 1400000000000,
            "freeBytes": 1600000000000,
            "volumeCount": 3
          }
        }"#;

        let parsed = parse_system_overview(payload).expect("sample payload should parse");

        assert_eq!(
            parsed,
            SystemOverview {
                windows: WindowsOverview {
                    product_name: Some("Windows 11 Pro".to_string()),
                    display_version: Some("24H2".to_string()),
                    build_number: Some("26100.8246".to_string()),
                },
                cpu: CpuOverview {
                    model: Some("AMD Ryzen 9 7945HX with Radeon Graphics".to_string()),
                    max_clock_mhz: Some(2500),
                    core_count: Some(16),
                    logical_processor_count: Some(32),
                },
                memory: MemoryOverview {
                    total_bytes: Some(68_719_476_736),
                    speed_mts: Some(5200),
                    used_slots: Some(2),
                    total_slots: Some(2),
                },
                gpu: Some(GpuOverview {
                    name: Some("AMD Radeon(TM) 610M".to_string()),
                    memory_bytes: Some(4_294_967_296),
                    driver_version: Some("32.0.11018.1000".to_string()),
                }),
                storage: StorageOverview {
                    total_bytes: Some(3_000_000_000_000),
                    used_bytes: Some(1_400_000_000_000),
                    free_bytes: Some(1_600_000_000_000),
                    volume_count: Some(3),
                },
            }
        );
    }

    #[test]
    fn parse_storage_only_system_overview_deserializes_contract_shape() {
        let payload = br#"{
          "windows": {
            "productName": null,
            "displayVersion": null,
            "buildNumber": null
          },
          "cpu": {
            "model": null,
            "maxClockMhz": null,
            "coreCount": null,
            "logicalProcessorCount": null
          },
          "memory": {
            "totalBytes": null,
            "speedMts": null,
            "usedSlots": null,
            "totalSlots": null
          },
          "gpu": null,
          "storage": {
            "totalBytes": 3000000000000,
            "usedBytes": 1400000000000,
            "freeBytes": 1600000000000,
            "volumeCount": 3
          }
        }"#;

        let parsed = parse_system_overview(payload).expect("storage-only payload should parse");

        assert_eq!(
            parsed,
            SystemOverview {
                windows: WindowsOverview {
                    product_name: None,
                    display_version: None,
                    build_number: None,
                },
                cpu: CpuOverview {
                    model: None,
                    max_clock_mhz: None,
                    core_count: None,
                    logical_processor_count: None,
                },
                memory: MemoryOverview {
                    total_bytes: None,
                    speed_mts: None,
                    used_slots: None,
                    total_slots: None,
                },
                gpu: None,
                storage: StorageOverview {
                    total_bytes: Some(3_000_000_000_000),
                    used_bytes: Some(1_400_000_000_000),
                    free_bytes: Some(1_600_000_000_000),
                    volume_count: Some(3),
                },
            }
        );
    }

    #[test]
    fn parse_system_overview_trims_bom_and_outer_whitespace() {
        let mut payload = b"  \r\n\xEF\xBB\xBF".to_vec();
        payload.extend_from_slice(
            br#"{
              "windows": {
                "productName": null,
                "displayVersion": null,
                "buildNumber": null
              },
              "cpu": {
                "model": null,
                "maxClockMhz": null,
                "coreCount": null,
                "logicalProcessorCount": null
              },
              "memory": {
                "totalBytes": null,
                "speedMts": null,
                "usedSlots": null,
                "totalSlots": null
              },
              "gpu": null,
              "storage": {
                "totalBytes": null,
                "usedBytes": null,
                "freeBytes": null,
                "volumeCount": null
              }
            }
          "#,
        );

        let parsed = parse_system_overview(&payload).expect("BOM-prefixed payload should parse");

        assert_eq!(parsed.gpu, None);
        assert_eq!(parsed.storage.free_bytes, None);
    }

    #[test]
    fn system_overview_scope_uses_lowercase_wire_values() {
        let full =
            serde_json::to_string(&SystemOverviewScope::Full).expect("full scope should serialize");
        let storage = serde_json::to_string(&SystemOverviewScope::Storage)
            .expect("storage scope should serialize");

        assert_eq!(full, "\"full\"");
        assert_eq!(storage, "\"storage\"");
    }

    #[test]
    fn system_overview_errors_use_stable_user_message() {
        let json_error = SystemOverviewQueryError::Json(
            serde_json::from_str::<SystemOverview>("{}")
                .expect_err("deserializing empty object should fail"),
        );

        assert_eq!(json_error.to_user_message(), SYSTEM_OVERVIEW_ERROR_MESSAGE);

        let spawn_error = SystemOverviewQueryError::Spawn(std::io::Error::other("spawn failed"));
        let io_error = SystemOverviewQueryError::Io(std::io::Error::other("read failed"));
        let command_error = SystemOverviewQueryError::CommandFailed {
            status: Some(1),
            stderr: "boom".to_string(),
        };
        let timeout_error = SystemOverviewQueryError::TimedOut {
            timeout_ms: HOST_QUERY_TIMEOUT_MS,
        };

        assert_eq!(spawn_error.to_user_message(), SYSTEM_OVERVIEW_ERROR_MESSAGE);
        assert_eq!(io_error.to_user_message(), SYSTEM_OVERVIEW_ERROR_MESSAGE);
        assert_eq!(
            command_error.to_user_message(),
            SYSTEM_OVERVIEW_ERROR_MESSAGE
        );
        assert_eq!(
            timeout_error.to_user_message(),
            HOST_COMMAND_TIMEOUT_MESSAGE
        );
        assert!(timeout_error.to_string().contains(SYSTEM_OVERVIEW_CONTEXT));
    }
}
