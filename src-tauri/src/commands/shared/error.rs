use serde::Serialize;
use wsl_core::WslError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CommandErrorDto {
    Wsl {
        code: WslCommandErrorCode,
        #[serde(rename = "wslCode", skip_serializing_if = "Option::is_none")]
        wsl_code: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        distro: Option<String>,
    },
    Message {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WslCommandErrorCode {
    InvalidArgument,
    FileNotFound,
    DistroNotFound,
    DiskResizeFailed,
    OperationNotPermitted,
    UnknownWslError,
    RegistryReadFailed,
    OutputParseFailed,
    WslCommandTimedOut,
    ProcessFailed,
    ProcessKilled,
    Cancelled,
}

pub(crate) fn map_command_error(err: WslError) -> CommandErrorDto {
    match err {
        WslError::InvalidArgument { .. } => wsl_error(WslCommandErrorCode::InvalidArgument),
        WslError::FileNotFound => wsl_error(WslCommandErrorCode::FileNotFound),
        WslError::DistroNotFound => wsl_error(WslCommandErrorCode::DistroNotFound),
        WslError::DiskResizeFailed => wsl_error(WslCommandErrorCode::DiskResizeFailed),
        WslError::OperationNotPermitted { distro } => {
            wsl_detail_error(WslCommandErrorCode::OperationNotPermitted, None, None, Some(distro))
        }
        WslError::UnknownWslError {
            code, raw_output, ..
        } => wsl_detail_error(
            WslCommandErrorCode::UnknownWslError,
            Some(code),
            non_empty(&raw_output),
            None,
        ),
        WslError::RegistryReadFailed { .. } => {
            wsl_error(WslCommandErrorCode::RegistryReadFailed)
        }
        WslError::OutputParseFailed { .. } => {
            wsl_error(WslCommandErrorCode::OutputParseFailed)
        }
        WslError::WslCommandTimedOut { .. } => {
            wsl_error(WslCommandErrorCode::WslCommandTimedOut)
        }
        WslError::ProcessFailed(_) => wsl_error(WslCommandErrorCode::ProcessFailed),
        WslError::ProcessKilled => wsl_error(WslCommandErrorCode::ProcessKilled),
        WslError::Cancelled => wsl_error(WslCommandErrorCode::Cancelled),
    }
}

pub(crate) fn message_command_error(message: impl Into<String>) -> CommandErrorDto {
    let message = message.into();
    CommandErrorDto::Message {
        message: non_empty(&message).unwrap_or(message),
    }
}

fn wsl_error(code: WslCommandErrorCode) -> CommandErrorDto {
    wsl_detail_error(code, None, None, None)
}

fn wsl_detail_error(
    code: WslCommandErrorCode,
    wsl_code: Option<String>,
    details: Option<String>,
    distro: Option<String>,
) -> CommandErrorDto {
    CommandErrorDto::Wsl {
        code,
        wsl_code,
        details,
        distro,
    }
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{map_command_error, message_command_error};
    use wsl_core::{WslCommandContext, WslError};

    #[test]
    fn serializes_expected_union_shapes() {
        for (err, code) in [
            (
                WslError::InvalidArgument {
                    context: WslCommandContext::Install,
                    raw_output: "bad args".to_string(),
                },
                "invalidArgument",
            ),
            (WslError::FileNotFound, "fileNotFound"),
            (WslError::DistroNotFound, "distroNotFound"),
            (WslError::DiskResizeFailed, "diskResizeFailed"),
            (
                WslError::RegistryReadFailed {
                    key: Some("HKCU\\...\\Lxss".to_string()),
                    detail: "missing".to_string(),
                },
                "registryReadFailed",
            ),
            (
                WslError::OutputParseFailed {
                    context: WslCommandContext::ListOnline,
                    detail: "missing header".to_string(),
                    raw_output: "raw".to_string(),
                },
                "outputParseFailed",
            ),
            (
                WslError::WslCommandTimedOut {
                    context: WslCommandContext::ListVerbose,
                },
                "wslCommandTimedOut",
            ),
            (
                WslError::ProcessFailed(std::io::Error::other("spawn failed")),
                "processFailed",
            ),
            (WslError::ProcessKilled, "processKilled"),
            (WslError::Cancelled, "cancelled"),
        ] {
            assert_eq!(
                serde_json::to_value(map_command_error(err)).expect("error should serialize"),
                json!({ "kind": "wsl", "code": code })
            );
        }

        assert_eq!(
            serde_json::to_value(map_command_error(WslError::OperationNotPermitted {
                distro: "docker-desktop".to_string(),
            }))
            .expect("protected error should serialize"),
            json!({
                "kind": "wsl",
                "code": "operationNotPermitted",
                "distro": "docker-desktop",
            })
        );
        assert_eq!(
            serde_json::to_value(map_command_error(WslError::UnknownWslError {
                context: WslCommandContext::Install,
                code: "exit-status:-1".to_string(),
                raw_output: "bad output".to_string(),
            }))
            .expect("unknown error should serialize"),
            json!({
                "kind": "wsl",
                "code": "unknownWslError",
                "wslCode": "exit-status:-1",
                "details": "bad output",
            })
        );

        assert_eq!(
            serde_json::to_value(message_command_error(" emit failed ".to_string()))
                .expect("message error should serialize"),
            json!({ "kind": "message", "message": "emit failed" })
        );
    }
}
