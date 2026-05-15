use crate::domain::error::{WslCommandContext, WslError};

const PROTECTED_DISTRO_NAMES: &[&str] = &["docker-desktop"];

pub(crate) fn validate_distro_input(
    context: WslCommandContext,
    distro: &str,
) -> Result<(), WslError> {
    validate_distro_name(context, distro)?;
    ensure_operation_permitted(context, distro)
}

pub(crate) fn validate_distro_name(
    context: WslCommandContext,
    distro: &str,
) -> Result<(), WslError> {
    if distro.trim().is_empty() {
        return Err(WslError::InvalidArgument {
            context,
            raw_output: "Distro name is required.".to_string(),
        });
    }

    if distro.chars().any(char::is_whitespace) {
        return Err(WslError::InvalidArgument {
            context,
            raw_output: format!(
                "Distro name `{distro}` contains whitespace. Project input rules do not allow spaces."
            ),
        });
    }

    Ok(())
}

pub(crate) fn ensure_operation_permitted(
    context: WslCommandContext,
    distro: &str,
) -> Result<(), WslError> {
    if is_protected_distro(distro)
        && matches!(
            context,
            WslCommandContext::Unregister
                | WslCommandContext::MoveDistro
                | WslCommandContext::ResizeDistro
        )
    {
        return Err(WslError::OperationNotPermitted {
            distro: distro.to_string(),
        });
    }

    Ok(())
}

fn is_protected_distro(distro: &str) -> bool {
    PROTECTED_DISTRO_NAMES
        .iter()
        .any(|protected| distro.eq_ignore_ascii_case(protected))
}

#[cfg(test)]
mod tests {
    use super::{ensure_operation_permitted, validate_distro_input, validate_distro_name};
    use crate::{WslCommandContext, WslError};

    #[test]
    fn distro_name_validation_rejects_blank_input() {
        let err = validate_distro_name(WslCommandContext::SetDefault, "   ")
            .expect_err("blank distro names should fail");

        match err {
            WslError::InvalidArgument {
                context,
                raw_output,
            } => {
                assert_eq!(context, WslCommandContext::SetDefault);
                assert!(raw_output.contains("required"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn distro_name_validation_rejects_whitespace() {
        let err = validate_distro_name(WslCommandContext::SetDefault, "My Ubuntu")
            .expect_err("whitespace should fail");

        match err {
            WslError::InvalidArgument {
                context,
                raw_output,
            } => {
                assert_eq!(context, WslCommandContext::SetDefault);
                assert!(raw_output.contains("contains whitespace"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn protected_distro_rejects_restricted_operations() {
        let err = ensure_operation_permitted(WslCommandContext::MoveDistro, "docker-desktop")
            .expect_err("docker-desktop move should fail");

        match err {
            WslError::OperationNotPermitted { distro } => {
                assert_eq!(distro, "docker-desktop");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn protected_distro_is_case_insensitive() {
        let err = validate_distro_input(WslCommandContext::ResizeDistro, "Docker-Desktop")
            .expect_err("protected distro matching should ignore ASCII case");

        match err {
            WslError::OperationNotPermitted { distro } => {
                assert_eq!(distro, "Docker-Desktop");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn protected_distro_allows_unrestricted_operations() {
        validate_distro_input(WslCommandContext::Export, "docker-desktop")
            .expect("export remains allowed for docker-desktop");
        validate_distro_input(WslCommandContext::SetDefault, "docker-desktop")
            .expect("set-default remains allowed for docker-desktop");
    }
}
