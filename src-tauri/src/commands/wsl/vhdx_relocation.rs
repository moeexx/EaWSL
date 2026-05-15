use std::{
    fs,
    path::{Path, PathBuf},
};

use wsl_core::ProgressState;

use crate::bridge::progress::{
    copy_progress_event, emit_transfer_progress, status_progress_event, ProgressEmitter,
    TransferProgressPhase,
};
use crate::services::system::filesystem::{self, FileCopyError};

const TARGET_DIRECTORY_EXISTS_MESSAGE: &str = "The target VHDX directory already exists.";
const VHDX_RELOCATION_FAILED_MESSAGE: &str = "Failed to relocate the VHDX file.";

#[derive(Debug)]
pub(crate) struct VhdxRelocation {
    pub(crate) final_vhdx: PathBuf,
    created_dir: Option<PathBuf>,
}

impl VhdxRelocation {
    fn in_place(source_vhdx: PathBuf) -> Self {
        Self {
            final_vhdx: source_vhdx,
            created_dir: None,
        }
    }

    fn copied(final_vhdx: PathBuf, created_dir: PathBuf) -> Self {
        Self {
            final_vhdx,
            created_dir: Some(created_dir),
        }
    }

    pub(crate) fn cleanup_failed_import(&self) {
        if self.created_dir.is_none() {
            return;
        }

        filesystem::cleanup_file_and_empty_dir(&self.final_vhdx, self.created_dir.as_deref());
    }
}

pub(crate) async fn prepare_vhdx_relocation<E>(
    emitter: E,
    event_name: &'static str,
    request_id: &str,
    distro: &str,
    source_vhdx: PathBuf,
    target_directory: Option<PathBuf>,
) -> Result<VhdxRelocation, String>
where
    E: ProgressEmitter,
{
    validate_vhdx_source(&source_vhdx)?;

    let Some(target_dir) = target_directory else {
        return Ok(VhdxRelocation::in_place(source_vhdx));
    };

    if target_dir.as_os_str().is_empty() {
        return Err(VHDX_RELOCATION_FAILED_MESSAGE.to_string());
    }

    if target_dir.exists() {
        return Err(TARGET_DIRECTORY_EXISTS_MESSAGE.to_string());
    }

    let file_name = source_vhdx
        .file_name()
        .ok_or_else(|| VHDX_RELOCATION_FAILED_MESSAGE.to_string())?;
    let target_vhdx = target_dir.join(file_name);

    let source_for_copy = source_vhdx.clone();
    let target_for_copy = target_vhdx.clone();
    let request_id = request_id.to_string();
    let distro = distro.to_string();

    let copy_result = tokio::task::spawn_blocking(move || {
        copy_vhdx_with_progress(
            emitter,
            event_name,
            &request_id,
            &distro,
            &source_for_copy,
            &target_for_copy,
        )
    })
    .await
    .map_err(|err| err.to_string())
    .and_then(|result| result);

    if let Err(err) = copy_result {
        filesystem::cleanup_file_and_empty_dir(&target_vhdx, Some(&target_dir));
        return Err(err);
    }

    Ok(VhdxRelocation::copied(target_vhdx, target_dir))
}

fn validate_vhdx_source(source_vhdx: &Path) -> Result<(), String> {
    let metadata =
        fs::metadata(source_vhdx).map_err(|_| VHDX_RELOCATION_FAILED_MESSAGE.to_string())?;

    if !metadata.is_file() || !has_vhdx_extension(source_vhdx) {
        return Err(VHDX_RELOCATION_FAILED_MESSAGE.to_string());
    }

    Ok(())
}

fn has_vhdx_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("vhdx"))
}

fn copy_vhdx_with_progress<E>(
    emitter: E,
    event_name: &'static str,
    request_id: &str,
    distro: &str,
    source: &Path,
    target: &Path,
) -> Result<(), String>
where
    E: ProgressEmitter,
{
    filesystem::copy_file_exclusive_with_progress(
        source,
        target,
        || {
            emit_transfer_progress(
                &emitter,
                event_name,
                request_id,
                distro,
                status_progress_event(TransferProgressPhase::Copying, ProgressState::Started),
            )?;
            Ok(())
        },
        |percent| {
            emit_transfer_progress(
                &emitter,
                event_name,
                request_id,
                distro,
                copy_progress_event(percent),
            )?;
            Ok(())
        },
    )
    .map_err(map_file_copy_error)?;

    Ok(())
}

fn map_file_copy_error(err: FileCopyError<String>) -> String {
    match err {
        FileCopyError::Io(_) => VHDX_RELOCATION_FAILED_MESSAGE.to_string(),
        FileCopyError::Progress(err) => err,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{Arc, Mutex},
    };

    use super::{
        has_vhdx_extension, prepare_vhdx_relocation, validate_vhdx_source,
        TARGET_DIRECTORY_EXISTS_MESSAGE, VHDX_RELOCATION_FAILED_MESSAGE,
    };
    use crate::bridge::progress::{
        DistroProgressEvent, ProgressEmitter, TransferProgressPhase, TransferProgressValue,
        TRANSFER_PROGRESS_EVENT,
    };
    use crate::commands::test_support::unique_temp_path as shared_unique_temp_path;
    use wsl_core::ProgressState;

    #[derive(Clone, Default)]
    struct RecordingEmitter {
        events: Arc<Mutex<Vec<(String, DistroProgressEvent)>>>,
    }

    impl RecordingEmitter {
        fn events(&self) -> Vec<(String, DistroProgressEvent)> {
            self.events.lock().expect("events mutex poisoned").clone()
        }
    }

    impl ProgressEmitter for RecordingEmitter {
        fn emit_progress(
            &self,
            event_name: &str,
            payload: DistroProgressEvent,
        ) -> Result<(), String> {
            self.events
                .lock()
                .expect("events mutex poisoned")
                .push((event_name.to_string(), payload));
            Ok(())
        }
    }

    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        shared_unique_temp_path("wsl-transfer", name)
    }

    #[test]
    fn vhdx_extension_check_is_case_insensitive() {
        assert!(has_vhdx_extension(std::path::Path::new("D:/WSL/ext4.VHDX")));
        assert!(!has_vhdx_extension(std::path::Path::new("D:/WSL/ext4.vhd")));
    }

    #[test]
    fn vhdx_source_validation_requires_file_and_extension() {
        let path = unique_temp_path("source").with_extension("vhdx");
        fs::write(&path, b"vhdx").expect("temp vhdx should be created");

        validate_vhdx_source(&path).expect("vhdx file should pass validation");

        fs::remove_file(&path).expect("temp vhdx should be removed");

        let err = validate_vhdx_source(&path).expect_err("missing file should fail");
        assert_eq!(err, VHDX_RELOCATION_FAILED_MESSAGE);
    }

    #[test]
    fn vhdx_source_validation_rejects_directory_with_vhdx_extension() {
        let path = unique_temp_path("source-dir").with_extension("vhdx");
        fs::create_dir_all(&path).expect("temp directory should be created");

        let err = validate_vhdx_source(&path).expect_err("directory should fail");

        fs::remove_dir_all(&path).expect("temp directory should be removed");

        assert_eq!(err, VHDX_RELOCATION_FAILED_MESSAGE);
    }

    #[tokio::test]
    async fn prepare_vhdx_relocation_copies_to_target_directory_and_emits_progress() {
        let source = unique_temp_path("relocation-source").with_extension("vhdx");
        let target_dir = unique_temp_path("relocation-target");
        fs::write(&source, b"vhdx-content").expect("temp source vhdx should be created");
        let emitter = RecordingEmitter::default();

        let relocation = prepare_vhdx_relocation(
            emitter.clone(),
            TRANSFER_PROGRESS_EVENT,
            "req-copy",
            "Ubuntu",
            source.clone(),
            Some(target_dir.clone()),
        )
        .await
        .expect("relocation should copy source vhdx");

        let expected_target = target_dir.join(source.file_name().expect("source should have name"));
        assert_eq!(relocation.final_vhdx, expected_target);
        assert_eq!(
            relocation.created_dir.as_deref(),
            Some(target_dir.as_path())
        );
        assert_eq!(
            fs::read(&relocation.final_vhdx).expect("copied vhdx should be readable"),
            b"vhdx-content"
        );
        assert_eq!(
            emitter
                .events()
                .into_iter()
                .map(|(_, payload)| payload.progress)
                .collect::<Vec<_>>(),
            vec![
                crate::bridge::progress::TransferProgressEvent {
                    phase: TransferProgressPhase::Copying,
                    value: TransferProgressValue::Status(ProgressState::Started),
                },
                crate::bridge::progress::TransferProgressEvent {
                    phase: TransferProgressPhase::Copying,
                    value: TransferProgressValue::Percent(100.0),
                },
            ]
        );

        fs::remove_dir_all(&target_dir).expect("target directory should be removed");
        fs::remove_file(&source).expect("source vhdx should be removed");
    }

    #[test]
    fn target_directory_conflict_message_is_stable() {
        assert_eq!(
            TARGET_DIRECTORY_EXISTS_MESSAGE,
            "The target VHDX directory already exists."
        );
    }
}
