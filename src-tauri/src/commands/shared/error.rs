use wsl_core::WslError;

pub(crate) fn map_command_error(err: WslError) -> String {
    err.to_user_message()
}

#[cfg(test)]
mod tests {
    use super::map_command_error;

    #[test]
    fn error_mapping_prefers_user_messages() {
        let err = map_command_error(wsl_core::WslError::RegistryReadFailed {
            key: Some("HKCU\\...\\Lxss".to_string()),
            detail: "missing".to_string(),
        });

        assert_eq!(err, "Failed to read WSL registry information.");
    }
}
