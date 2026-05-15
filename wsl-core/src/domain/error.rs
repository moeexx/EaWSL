use std::fmt;

/// Command context for mapping `Wsl/<CODE>` to domain errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WslCommandContext {
    Version,
    ListVerbose,
    ListOnline,
    Unregister,
    Terminate,
    Shutdown,
    SetDefault,
    MoveDistro,
    ResizeDistro,
    ImportInPlace,
    Install,
    Import,
    Export,
}

/// Rust Core error model.
#[derive(Debug)]
pub enum WslError {
    InvalidArgument {
        context: WslCommandContext,
        raw_output: String,
    },
    FileNotFound,
    DistroNotFound,
    DiskResizeFailed,
    OperationNotPermitted {
        distro: String,
    },
    UnknownWslError {
        context: WslCommandContext,
        code: String,
    },
    RegistryReadFailed {
        key: Option<String>,
        detail: String,
    },
    OutputParseFailed {
        context: WslCommandContext,
        detail: String,
        raw_output: String,
    },
    WslCommandTimedOut {
        context: WslCommandContext,
    },
    ProcessFailed(std::io::Error),
    ProcessKilled,
    Cancelled,
}

impl WslCommandContext {
    fn as_str(self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::ListVerbose => "list_verbose",
            Self::ListOnline => "list_online",
            Self::Unregister => "unregister",
            Self::Terminate => "terminate",
            Self::Shutdown => "shutdown",
            Self::SetDefault => "set_default",
            Self::MoveDistro => "move_distro",
            Self::ResizeDistro => "resize_distro",
            Self::ImportInPlace => "import_in_place",
            Self::Install => "install",
            Self::Import => "import",
            Self::Export => "export",
        }
    }
}

impl WslError {
    /// Stable user-facing message.
    pub fn to_user_message(&self) -> String {
        match self {
            Self::InvalidArgument { .. } => "The WSL command arguments are invalid.".to_string(),
            Self::FileNotFound => "The specified file was not found.".to_string(),
            Self::DistroNotFound => "The specified distro was not found.".to_string(),
            Self::DiskResizeFailed => "Failed to resize the distro disk.".to_string(),
            Self::OperationNotPermitted { distro } => {
                format!("The requested operation is not permitted for distro `{distro}`.")
            }
            Self::UnknownWslError { code, .. } => format!("The WSL command failed: {code}."),
            Self::RegistryReadFailed { .. } => {
                "Failed to read WSL registry information.".to_string()
            }
            Self::OutputParseFailed { .. } => "Failed to parse WSL command output.".to_string(),
            Self::WslCommandTimedOut { .. } => {
                "The WSL command timed out before a stable result was available.".to_string()
            }
            Self::ProcessFailed(_) => "Failed to start the WSL command.".to_string(),
            Self::ProcessKilled => "The WSL command exited without a status code.".to_string(),
            Self::Cancelled => "The operation was cancelled by the user.".to_string(),
        }
    }
}

impl fmt::Display for WslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument {
                context,
                raw_output,
            } => write!(
                f,
                "invalid argument while running {}: {}",
                context.as_str(),
                summarize_output(raw_output)
            ),
            Self::FileNotFound => write!(f, "file not found"),
            Self::DistroNotFound => write!(f, "distro not found"),
            Self::DiskResizeFailed => write!(f, "disk resize failed"),
            Self::OperationNotPermitted { distro } => {
                write!(f, "operation not permitted for distro {}", distro)
            }
            Self::UnknownWslError { context, code } => {
                write!(f, "unknown WSL error in {}: {}", context.as_str(), code)
            }
            Self::RegistryReadFailed { key, detail } => write!(
                f,
                "failed to read registry{}: {}",
                key.as_deref()
                    .map(|value| format!(" key {}", value))
                    .unwrap_or_default(),
                detail
            ),
            Self::OutputParseFailed {
                context,
                detail,
                raw_output,
            } => write!(
                f,
                "failed to parse {} output: {} ({})",
                context.as_str(),
                detail,
                summarize_output(raw_output)
            ),
            Self::WslCommandTimedOut { context } => {
                write!(f, "wsl command timed out in {}", context.as_str())
            }
            Self::ProcessFailed(err) => write!(f, "failed to start process: {}", err),
            Self::ProcessKilled => write!(f, "process exited without a status code"),
            Self::Cancelled => write!(f, "process was cancelled"),
        }
    }
}

impl std::error::Error for WslError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ProcessFailed(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(test)]
fn map_wsl_error(ctx: WslCommandContext, code: &str) -> WslError {
    map_wsl_error_with_output(ctx, code, String::new())
}

pub(crate) fn map_wsl_error_with_output(
    ctx: WslCommandContext,
    code: &str,
    raw_output: String,
) -> WslError {
    let normalized = code.trim();

    match normalized {
        "Wsl/E_INVALIDARG" => WslError::InvalidArgument {
            context: ctx,
            raw_output,
        },
        "Wsl/Service/WSL_E_DISTRO_NOT_FOUND"
            if matches!(
                ctx,
                WslCommandContext::SetDefault
                    | WslCommandContext::Terminate
                    | WslCommandContext::Unregister
                    | WslCommandContext::MoveDistro
                    | WslCommandContext::ResizeDistro
                    | WslCommandContext::Export
            ) =>
        {
            WslError::DistroNotFound
        }
        "Wsl/ERROR_FILE_NOT_FOUND"
            if matches!(
                ctx,
                WslCommandContext::Import | WslCommandContext::ImportInPlace
            ) =>
        {
            WslError::FileNotFound
        }
        "Wsl/Service/E_FAIL" if matches!(ctx, WslCommandContext::ResizeDistro) => {
            WslError::DiskResizeFailed
        }
        _ => WslError::UnknownWslError {
            context: ctx,
            code: normalized.to_string(),
        },
    }
}

fn summarize_output(raw_output: &str) -> String {
    const LIMIT: usize = 160;

    let normalized = raw_output
        .trim()
        .replace("\r\n", " | ")
        .replace(['\n', '\r'], " | ");

    if normalized.is_empty() {
        return "<empty>".to_string();
    }

    if normalized.len() <= LIMIT {
        return normalized;
    }

    let mut summary = normalized;
    summary.truncate(LIMIT);
    summary.push_str("...");
    summary
}

#[cfg(test)]
mod tests {
    use super::{map_wsl_error, WslCommandContext, WslError};

    #[test]
    fn map_invalid_argument_to_variant() {
        let err = map_wsl_error(WslCommandContext::ListVerbose, "Wsl/E_INVALIDARG");
        match err {
            WslError::InvalidArgument {
                context,
                raw_output,
            } => {
                assert_eq!(context, WslCommandContext::ListVerbose);
                assert!(raw_output.is_empty());
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn map_distro_not_found_by_context() {
        let err = map_wsl_error(
            WslCommandContext::Export,
            "Wsl/Service/WSL_E_DISTRO_NOT_FOUND",
        );
        assert!(matches!(err, WslError::DistroNotFound));
    }

    #[test]
    fn map_file_not_found_by_context() {
        let err = map_wsl_error(WslCommandContext::Import, "Wsl/ERROR_FILE_NOT_FOUND");
        assert!(matches!(err, WslError::FileNotFound));
    }

    #[test]
    fn map_resize_failure_by_context() {
        let err = map_wsl_error(WslCommandContext::ResizeDistro, "Wsl/Service/E_FAIL");
        assert!(matches!(err, WslError::DiskResizeFailed));
    }

    #[test]
    fn map_unknown_error_falls_back() {
        let err = map_wsl_error(WslCommandContext::Version, "Wsl/UNKNOWN_CODE");
        match err {
            WslError::UnknownWslError { context, code } => {
                assert_eq!(context, WslCommandContext::Version);
                assert_eq!(code, "Wsl/UNKNOWN_CODE");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn user_message_and_display_are_actionable() {
        let err = WslError::OutputParseFailed {
            context: WslCommandContext::ListOnline,
            detail: "missing header".to_string(),
            raw_output: "NAME".to_string(),
        };

        assert_eq!(err.to_user_message(), "Failed to parse WSL command output.");

        let rendered = err.to_string();
        assert!(rendered.contains("list_online"));
        assert!(rendered.contains("missing header"));
        assert!(rendered.contains("NAME"));
    }

    #[test]
    fn timeout_variant_uses_stable_message_and_context() {
        let wsl_timeout = WslError::WslCommandTimedOut {
            context: WslCommandContext::ListVerbose,
        };

        assert_eq!(
            wsl_timeout.to_user_message(),
            "The WSL command timed out before a stable result was available."
        );
        assert!(wsl_timeout.to_string().contains("list_verbose"));
    }
}
