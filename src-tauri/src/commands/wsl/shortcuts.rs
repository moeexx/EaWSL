use std::{
    env, io,
    path::PathBuf,
    process::{Child, Command},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x00000010;

const DISTRO_REQUIRED: &str = "Distro name is required.";
const TERMINAL_LAUNCH_FAILED: &str = "Failed to open distro terminal.";
const EXPLORER_LAUNCH_FAILED: &str = "Failed to open distro in Explorer.";
const VSCODE_LAUNCH_FAILED: &str = "Failed to open distro in VS Code.";

#[tauri::command]
pub async fn open_distro_terminal(distro: String) -> Result<(), String> {
    let distro = normalize_distro_name(&distro)?;

    spawn_first([
        build_distro_terminal_command(&distro),
        build_distro_terminal_fallback_command(&distro),
    ])
    .map(|_| ())
    .map_err(|_| TERMINAL_LAUNCH_FAILED.to_string())
}

#[tauri::command]
pub async fn open_distro_explorer(distro: String) -> Result<(), String> {
    let distro = normalize_distro_name(&distro)?;

    spawn_command(build_distro_explorer_command(&distro))
        .map(|_| ())
        .map_err(|_| EXPLORER_LAUNCH_FAILED.to_string())
}

#[tauri::command]
pub async fn open_distro_vscode(distro: String) -> Result<(), String> {
    let distro = normalize_distro_name(&distro)?;

    let mut commands = vec![
        build_distro_vscode_command(&distro),
        build_distro_vscode_fallback_command(&distro),
    ];
    commands.extend(
        vscode_exe_candidates()
            .into_iter()
            .map(|path| build_distro_vscode_exe_command(path, &distro)),
    );

    spawn_first(commands)
        .map(|_| ())
        .map_err(|_| VSCODE_LAUNCH_FAILED.to_string())
}

fn normalize_distro_name(distro: &str) -> Result<String, String> {
    let trimmed = distro.trim();
    if trimmed.is_empty() {
        return Err(DISTRO_REQUIRED.to_string());
    }

    Ok(trimmed.to_string())
}

fn spawn_command(mut command: Command) -> Result<Child, io::Error> {
    command.spawn()
}

fn spawn_first(commands: impl IntoIterator<Item = Command>) -> Result<Child, io::Error> {
    let mut last_error = None;

    for command in commands {
        match spawn_command(command) {
            Ok(child) => return Ok(child),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "No launch command candidates.")
    }))
}

fn build_distro_terminal_command(distro: &str) -> Command {
    let mut command = Command::new("wt.exe");
    command.args(["new-tab", "wsl.exe", "-d", distro]);
    command
}

fn build_distro_terminal_fallback_command(distro: &str) -> Command {
    let mut command = Command::new("wsl.exe");
    command.args(["-d", distro]);
    #[cfg(windows)]
    command.creation_flags(CREATE_NEW_CONSOLE);
    command
}

fn build_distro_explorer_command(distro: &str) -> Command {
    let mut command = Command::new("explorer.exe");
    command.arg(format!("\\\\wsl$\\{}\\home", distro));
    command
}

fn build_distro_vscode_command(distro: &str) -> Command {
    let mut command = Command::new("code");
    command.args(["--remote", &format!("wsl+{}", distro)]);
    command
}

fn build_distro_vscode_fallback_command(distro: &str) -> Command {
    let mut command = Command::new("code.cmd");
    command.args(["--remote", &format!("wsl+{}", distro)]);
    command
}

fn build_distro_vscode_exe_command(path: PathBuf, distro: &str) -> Command {
    let mut command = Command::new(path);
    command.args(["--remote", &format!("wsl+{}", distro)]);
    command
}

fn vscode_exe_candidates() -> Vec<PathBuf> {
    [env::var_os("LOCALAPPDATA").map(|root| {
        PathBuf::from(root)
            .join("Programs")
            .join("Microsoft VS Code")
            .join("Code.exe")
    })]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        build_distro_explorer_command, build_distro_terminal_command,
        build_distro_terminal_fallback_command, build_distro_vscode_command,
        build_distro_vscode_exe_command, build_distro_vscode_fallback_command,
        normalize_distro_name,
    };
    use std::path::PathBuf;

    #[test]
    fn distro_terminal_command_uses_windows_terminal_new_tab() {
        let command = build_distro_terminal_command("debian");

        assert_eq!(command.get_program(), "wt.exe");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["new-tab", "wsl.exe", "-d", "debian"]
        );
    }

    #[test]
    fn distro_terminal_fallback_command_uses_wsl_directly() {
        let command = build_distro_terminal_fallback_command("debian");

        assert_eq!(command.get_program(), "wsl.exe");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["-d", "debian"]
        );
    }

    #[test]
    fn distro_explorer_command_uses_wsl_share_home_path() {
        let command = build_distro_explorer_command("debian");

        assert_eq!(command.get_program(), "explorer.exe");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec![r"\\wsl$\debian\home"]
        );
    }

    #[test]
    fn distro_vscode_command_uses_remote_wsl_authority() {
        let command = build_distro_vscode_command("Debian");

        assert_eq!(command.get_program(), "code");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["--remote", "wsl+Debian"]
        );
    }

    #[test]
    fn distro_vscode_fallback_command_uses_windows_cmd_launcher() {
        let command = build_distro_vscode_fallback_command("Debian");

        assert_eq!(command.get_program(), "code.cmd");
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["--remote", "wsl+Debian"]
        );
    }

    #[test]
    fn distro_vscode_exe_command_uses_explicit_code_exe_path() {
        let path = PathBuf::from(r"C:\VSCode\Code.exe");
        let command = build_distro_vscode_exe_command(path.clone(), "Debian");

        assert_eq!(command.get_program(), path.as_os_str());
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["--remote", "wsl+Debian"]
        );
    }

    #[test]
    fn shortcut_distro_name_rejects_blank_and_trims_value() {
        assert_eq!(normalize_distro_name(" debian ").unwrap(), "debian");
        assert!(normalize_distro_name(" ").is_err());
    }
}
