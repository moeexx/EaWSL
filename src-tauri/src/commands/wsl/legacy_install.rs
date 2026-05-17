use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

const LEGACY_INSTALL_LAUNCH_FAILED: &str = "Failed to open legacy install terminal.";

#[tauri::command]
pub async fn launch_legacy_install_terminal(distro: String) -> Result<(), String> {
    validate_legacy_distro_name(&distro)?;
    launch_legacy_install_command(&distro).map(|_| ())
}

fn validate_legacy_distro_name(distro: &str) -> Result<(), String> {
    if distro.trim().is_empty() {
        return Err("Distro name is required.".to_string());
    }

    if distro.chars().any(char::is_whitespace) {
        return Err("Distro name cannot contain whitespace.".to_string());
    }

    Ok(())
}

fn launch_legacy_install_command(distro: &str) -> Result<std::process::Child, String> {
    build_legacy_install_command(distro)
        .spawn()
        .map_err(|_| LEGACY_INSTALL_LAUNCH_FAILED.to_string())
}

fn build_legacy_install_command(distro: &str) -> Command {
    let mut command = Command::new("wsl.exe");
    command.args(["--install", distro]);
    #[cfg(windows)]
    command.creation_flags(CREATE_NEW_CONSOLE);
    command
}

#[cfg(test)]
mod tests {
    use super::{build_legacy_install_command, validate_legacy_distro_name};

    #[test]
    fn legacy_install_command_uses_interactive_wsl_install_args() {
        let command = build_legacy_install_command("Oraclelinux_7_9");

        assert_eq!(command.get_program(), "wsl.exe");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["--install", "Oraclelinux_7_9"]
        );
    }

    #[test]
    fn legacy_install_distro_name_rejects_blank_and_whitespace() {
        assert!(validate_legacy_distro_name("Oraclelinux_7_9").is_ok());
        assert!(validate_legacy_distro_name(" ").is_err());
        assert!(validate_legacy_distro_name("Oracle Linux").is_err());
    }
}
