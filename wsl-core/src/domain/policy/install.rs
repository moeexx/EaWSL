use crate::domain::error::{WslCommandContext, WslError};
use crate::domain::policy::distro::validate_distro_name;
use crate::domain::value::install::InstallOptions;

pub(crate) fn validate_install_options(opts: &InstallOptions) -> Result<(), WslError> {
    if let Some(name) = opts.name.as_deref() {
        validate_distro_name(WslCommandContext::Install, name)?;
    }

    if opts.fixed_vhd && opts.vhd_size.is_none() {
        return Err(WslError::InvalidArgument {
            context: WslCommandContext::Install,
            raw_output: "Option `--fixed-vhd` requires `vhd_size`.".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_install_options;
    use crate::{InstallOptions, WslCommandContext, WslError};

    #[test]
    fn fixed_vhd_requires_vhd_size() {
        let opts = InstallOptions {
            name: None,
            location: None,
            vhd_size: None,
            fixed_vhd: true,
        };

        let err = validate_install_options(&opts).expect_err("missing vhd_size should fail");
        match err {
            WslError::InvalidArgument {
                context,
                raw_output,
            } => {
                assert_eq!(context, WslCommandContext::Install);
                assert!(raw_output.contains("fixed-vhd"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn install_name_accepts_project_valid_names() {
        let opts = InstallOptions {
            name: Some("custom-ubuntu".to_string()),
            location: None,
            vhd_size: None,
            fixed_vhd: false,
        };

        validate_install_options(&opts).expect("valid install name should pass");
    }

    #[test]
    fn install_name_reuses_distro_name_validation_rules() {
        let opts = InstallOptions {
            name: Some("My Ubuntu".to_string()),
            location: None,
            vhd_size: None,
            fixed_vhd: false,
        };

        let err =
            validate_install_options(&opts).expect_err("whitespace in install name should fail");
        match err {
            WslError::InvalidArgument {
                context,
                raw_output,
            } => {
                assert_eq!(context, WslCommandContext::Install);
                assert!(raw_output.contains("contains whitespace"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
