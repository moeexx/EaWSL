use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use serde::Serialize;

const PATH_VOLUME_SPACE_ERROR_MESSAGE: &str = "Failed to read volume free space information.";
const FILE_SYSTEM_PROBE_ERROR_MESSAGE: &str = "Failed to read filesystem information.";
const FILE_SIZE_ERROR_MESSAGE: &str = "Failed to read file size information.";
const COPY_BUFFER_SIZE: usize = 8 * 1024 * 1024;

#[link(name = "Kernel32")]
unsafe extern "system" {
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available_to_caller: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;

    fn GetVolumePathNameW(
        lpsz_file_name: *const u16,
        lpsz_volume_path_name: *mut u16,
        cch_buffer_length: u32,
    ) -> i32;
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PathVolumeSpace {
    pub volume_root: String,
    pub free_bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemPathProbe {
    pub exists: bool,
    pub is_file: bool,
    pub is_dir: bool,
    pub direct_child_count: Option<u32>,
    pub child_count_limit_exceeded: bool,
    pub direct_vhdx_file_count: Option<u32>,
    pub has_direct_children: Option<bool>,
}

#[derive(Debug)]
enum PathVolumeSpaceQueryError {
    Join(String),
    CurrentDir(std::io::Error),
    InvalidPath,
    AnchorNotFound(PathBuf),
    VolumeRoot {
        path: PathBuf,
        source: std::io::Error,
    },
    FreeSpace {
        volume_root: PathBuf,
        source: std::io::Error,
    },
}

impl PathVolumeSpaceQueryError {
    fn to_user_message(&self) -> String {
        PATH_VOLUME_SPACE_ERROR_MESSAGE.to_string()
    }
}

impl std::fmt::Display for PathVolumeSpaceQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Join(detail) => write!(f, "failed to join path volume-space task: {detail}"),
            Self::CurrentDir(source) => {
                write!(f, "failed to resolve current working directory: {source}")
            }
            Self::InvalidPath => write!(f, "path is required"),
            Self::AnchorNotFound(path) => {
                write!(
                    f,
                    "failed to resolve an existing storage anchor for {}",
                    path.display()
                )
            }
            Self::VolumeRoot { path, source } => {
                write!(
                    f,
                    "failed to resolve volume root for {}: {source}",
                    path.display()
                )
            }
            Self::FreeSpace {
                volume_root,
                source,
            } => {
                write!(
                    f,
                    "failed to query free space for {}: {source}",
                    volume_root.display()
                )
            }
        }
    }
}

#[derive(Debug)]
enum FileSystemProbeError {
    Join(String),
    InvalidPath,
    Metadata {
        path: PathBuf,
        source: std::io::Error,
    },
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
    ReadEntry {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl FileSystemProbeError {
    fn to_user_message(&self) -> String {
        FILE_SYSTEM_PROBE_ERROR_MESSAGE.to_string()
    }
}

impl std::fmt::Display for FileSystemProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Join(detail) => write!(f, "failed to join filesystem probe task: {detail}"),
            Self::InvalidPath => write!(f, "path is required"),
            Self::Metadata { path, source } => {
                write!(
                    f,
                    "failed to read metadata for {}: {source}",
                    path.display()
                )
            }
            Self::ReadDir { path, source } => {
                write!(f, "failed to read directory {}: {source}", path.display())
            }
            Self::ReadEntry { path, source } => {
                write!(
                    f,
                    "failed to read directory entry under {}: {source}",
                    path.display()
                )
            }
        }
    }
}

#[derive(Debug)]
enum FileSizeProbeError {
    Join(String),
    Metadata {
        path: PathBuf,
        source: std::io::Error,
    },
    NotAFile(PathBuf),
}

#[derive(Debug)]
pub(crate) enum FileCopyError<E> {
    Io(io::Error),
    Progress(E),
}

impl<E: fmt::Display> fmt::Display for FileCopyError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Progress(error) => write!(formatter, "{error}"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum WriteTextError {
    CreateDirectory { path: PathBuf, source: io::Error },
    Write { path: PathBuf, source: io::Error },
}

impl FileSizeProbeError {
    fn to_user_message(&self) -> String {
        FILE_SIZE_ERROR_MESSAGE.to_string()
    }
}

impl std::fmt::Display for FileSizeProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Join(detail) => write!(f, "failed to join file size task: {detail}"),
            Self::Metadata { path, source } => {
                write!(
                    f,
                    "failed to read metadata for {}: {source}",
                    path.display()
                )
            }
            Self::NotAFile(path) => write!(f, "{} is not a file", path.display()),
        }
    }
}

pub async fn get_path_volume_space(path: String) -> Result<PathVolumeSpace, String> {
    read_path_volume_space(PathBuf::from(path))
        .await
        .map_err(|err| err.to_user_message())
}

pub async fn probe_file_system_path(
    path: String,
    child_limit: Option<u32>,
) -> Result<FileSystemPathProbe, String> {
    probe_file_system_path_async(PathBuf::from(path), child_limit)
        .await
        .map_err(|err| err.to_user_message())
}

pub async fn get_file_size(path: String) -> Result<u64, String> {
    probe_file_size(PathBuf::from(path))
        .await
        .map_err(|err| err.to_user_message())
}

pub(crate) fn copy_file_exclusive_with_progress<E>(
    source: &Path,
    target: &Path,
    mut on_started: impl FnMut() -> Result<(), E>,
    mut on_percent_progress: impl FnMut(f32) -> Result<(), E>,
) -> Result<(), FileCopyError<E>> {
    let mut source_file = File::open(source).map_err(FileCopyError::Io)?;
    let total = source_file.metadata().map_err(FileCopyError::Io)?.len();
    ensure_parent_dir(target).map_err(FileCopyError::Io)?;

    let target_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(FileCopyError::Io)?;

    match copy_open_files(
        &mut source_file,
        target_file,
        total,
        &mut on_started,
        &mut on_percent_progress,
    ) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(target);
            Err(error)
        }
    }
}

pub(crate) fn write_text_creating_parent(path: &Path, content: &str) -> Result<(), WriteTextError> {
    if let Some(parent) = parent_dir(path) {
        fs::create_dir_all(parent).map_err(|source| WriteTextError::CreateDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(path, content).map_err(|source| WriteTextError::Write {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn cleanup_file_and_empty_dir(file: &Path, dir: Option<&Path>) {
    let _ = fs::remove_file(file);

    if let Some(dir) = dir {
        let _ = fs::remove_dir(dir);
    }
}

async fn read_path_volume_space(
    path: PathBuf,
) -> Result<PathVolumeSpace, PathVolumeSpaceQueryError> {
    tokio::task::spawn_blocking(move || read_path_volume_space_blocking(&path))
        .await
        .map_err(|err| PathVolumeSpaceQueryError::Join(err.to_string()))?
}

fn read_path_volume_space_blocking(
    path: &Path,
) -> Result<PathVolumeSpace, PathVolumeSpaceQueryError> {
    let anchor = resolve_storage_anchor(path)?;
    let volume_root = resolve_volume_root_windows(&anchor)?;
    let free_bytes = query_volume_free_space_windows(&volume_root)?;

    Ok(PathVolumeSpace {
        volume_root: volume_root.to_string_lossy().into_owned(),
        free_bytes,
    })
}

fn resolve_storage_anchor(path: &Path) -> Result<PathBuf, PathVolumeSpaceQueryError> {
    if path.as_os_str().is_empty() {
        return Err(PathVolumeSpaceQueryError::InvalidPath);
    }

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(PathVolumeSpaceQueryError::CurrentDir)?
            .join(path)
    };

    for ancestor in absolute_path.ancestors() {
        if ancestor.as_os_str().is_empty() || !ancestor.exists() {
            continue;
        }

        if ancestor.is_dir() {
            return Ok(ancestor.to_path_buf());
        }

        if let Some(parent) = ancestor.parent() {
            return Ok(parent.to_path_buf());
        }

        return Ok(ancestor.to_path_buf());
    }

    Err(PathVolumeSpaceQueryError::AnchorNotFound(absolute_path))
}

fn resolve_volume_root_windows(path: &Path) -> Result<PathBuf, PathVolumeSpaceQueryError> {
    let encoded = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let mut buffer = vec![0_u16; 32_768];

    let ok =
        unsafe { GetVolumePathNameW(encoded.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32) };

    if ok == 0 {
        return Err(PathVolumeSpaceQueryError::VolumeRoot {
            path: path.to_path_buf(),
            source: std::io::Error::last_os_error(),
        });
    }

    let len = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());

    if len == 0 {
        return Err(PathVolumeSpaceQueryError::VolumeRoot {
            path: path.to_path_buf(),
            source: std::io::Error::from_raw_os_error(0),
        });
    }

    Ok(PathBuf::from(String::from_utf16_lossy(&buffer[..len])))
}

fn query_volume_free_space_windows(path: &Path) -> Result<u64, PathVolumeSpaceQueryError> {
    let encoded = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let mut free_bytes = 0_u64;

    let ok = unsafe {
        GetDiskFreeSpaceExW(
            encoded.as_ptr(),
            &mut free_bytes,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    if ok == 0 {
        return Err(PathVolumeSpaceQueryError::FreeSpace {
            volume_root: path.to_path_buf(),
            source: std::io::Error::last_os_error(),
        });
    }

    Ok(free_bytes)
}

async fn probe_file_system_path_async(
    path: PathBuf,
    child_limit: Option<u32>,
) -> Result<FileSystemPathProbe, FileSystemProbeError> {
    tokio::task::spawn_blocking(move || probe_file_system_path_blocking(&path, child_limit))
        .await
        .map_err(|err| FileSystemProbeError::Join(err.to_string()))?
}

fn probe_file_system_path_blocking(
    path: &Path,
    child_limit: Option<u32>,
) -> Result<FileSystemPathProbe, FileSystemProbeError> {
    if path.as_os_str().is_empty() {
        return Err(FileSystemProbeError::InvalidPath);
    }

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FileSystemPathProbe {
                exists: false,
                is_file: false,
                is_dir: false,
                direct_child_count: None,
                child_count_limit_exceeded: false,
                direct_vhdx_file_count: None,
                has_direct_children: None,
            });
        }
        Err(source) => {
            return Err(FileSystemProbeError::Metadata {
                path: path.to_path_buf(),
                source,
            });
        }
    };

    let is_file = metadata.is_file();
    let is_dir = metadata.is_dir();
    if !is_dir {
        return Ok(FileSystemPathProbe {
            exists: true,
            is_file,
            is_dir,
            direct_child_count: None,
            child_count_limit_exceeded: false,
            direct_vhdx_file_count: None,
            has_direct_children: None,
        });
    }

    let directory = fs::read_dir(path).map_err(|source| FileSystemProbeError::ReadDir {
        path: path.to_path_buf(),
        source,
    })?;
    let limit = child_limit.unwrap_or(u32::MAX);
    let mut direct_child_count = 0_u32;
    let mut direct_vhdx_file_count = 0_u32;
    let mut child_count_limit_exceeded = false;

    for entry in directory {
        let entry = entry.map_err(|source| FileSystemProbeError::ReadEntry {
            path: path.to_path_buf(),
            source,
        })?;

        if direct_child_count >= limit {
            child_count_limit_exceeded = true;
            break;
        }

        direct_child_count += 1;

        if entry
            .file_type()
            .map_err(|source| FileSystemProbeError::ReadEntry {
                path: path.to_path_buf(),
                source,
            })?
            .is_file()
            && has_vhdx_extension(&entry.path())
        {
            direct_vhdx_file_count += 1;
        }
    }

    Ok(FileSystemPathProbe {
        exists: true,
        is_file,
        is_dir,
        direct_child_count: Some(direct_child_count),
        child_count_limit_exceeded,
        direct_vhdx_file_count: Some(direct_vhdx_file_count),
        has_direct_children: Some(direct_child_count > 0 || child_count_limit_exceeded),
    })
}

fn has_vhdx_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("vhdx"))
}

async fn probe_file_size(path: PathBuf) -> Result<u64, FileSizeProbeError> {
    tokio::task::spawn_blocking(move || probe_file_size_blocking(&path))
        .await
        .map_err(|err| FileSizeProbeError::Join(err.to_string()))?
}

fn probe_file_size_blocking(path: &Path) -> Result<u64, FileSizeProbeError> {
    let metadata = fs::metadata(path).map_err(|source| FileSizeProbeError::Metadata {
        path: path.to_path_buf(),
        source,
    })?;

    if !metadata.is_file() {
        return Err(FileSizeProbeError::NotAFile(path.to_path_buf()));
    }

    Ok(metadata.len())
}

fn ensure_parent_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = parent_dir(path) {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

fn parent_dir(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

fn copy_open_files<E>(
    source_file: &mut File,
    mut target_file: File,
    total: u64,
    on_started: &mut impl FnMut() -> Result<(), E>,
    on_percent_progress: &mut impl FnMut(f32) -> Result<(), E>,
) -> Result<(), FileCopyError<E>> {
    let mut buffer = vec![0_u8; COPY_BUFFER_SIZE];
    let mut copied = 0_u64;

    on_started().map_err(FileCopyError::Progress)?;

    loop {
        let read = source_file.read(&mut buffer).map_err(FileCopyError::Io)?;
        if read == 0 {
            break;
        }

        target_file
            .write_all(&buffer[..read])
            .map_err(FileCopyError::Io)?;
        copied += read as u64;
        on_percent_progress(copy_percent(copied, total)).map_err(FileCopyError::Progress)?;
    }

    target_file.flush().map_err(FileCopyError::Io)?;
    drop(target_file);

    if total == 0 {
        on_percent_progress(100.0).map_err(FileCopyError::Progress)?;
    }

    Ok(())
}

fn copy_percent(copied_bytes: u64, total_bytes: u64) -> f32 {
    if total_bytes == 0 {
        100.0
    } else {
        (copied_bytes as f32 / total_bytes as f32 * 100.0).min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, fs, io, path::Path};

    use super::{
        copy_file_exclusive_with_progress, probe_file_size_blocking,
        probe_file_system_path_blocking, read_path_volume_space_blocking, resolve_storage_anchor,
        write_text_creating_parent, FileCopyError, FileSizeProbeError, FileSystemProbeError,
        PathVolumeSpaceQueryError, FILE_SIZE_ERROR_MESSAGE, FILE_SYSTEM_PROBE_ERROR_MESSAGE,
        PATH_VOLUME_SPACE_ERROR_MESSAGE,
    };
    use crate::commands::test_support::unique_temp_path as shared_unique_temp_path;

    fn unique_temp_path(group: &str, name: &str) -> std::path::PathBuf {
        shared_unique_temp_path(group, name)
    }

    #[test]
    fn resolve_storage_anchor_falls_back_to_existing_parent() {
        let anchor = unique_temp_path("volume-space", "anchor");
        let missing_child = anchor.join("nested").join("missing");

        fs::create_dir_all(&anchor).expect("temp anchor directory should be created");

        let resolved = resolve_storage_anchor(&missing_child)
            .expect("existing parent should be used as storage anchor");

        fs::remove_dir_all(&anchor).expect("temp anchor directory should be removed");

        assert_eq!(resolved, anchor);
    }

    #[test]
    fn path_volume_space_reader_returns_volume_root_and_positive_bytes() {
        let volume_space = read_path_volume_space_blocking(&std::env::temp_dir())
            .expect("temp directory volume space should be readable");

        assert!(!volume_space.volume_root.is_empty());
        assert!(volume_space.free_bytes > 0);
    }

    #[test]
    fn path_volume_space_errors_use_stable_user_message() {
        let err =
            read_path_volume_space_blocking(Path::new("")).expect_err("empty path should fail");

        assert_eq!(err.to_user_message(), PATH_VOLUME_SPACE_ERROR_MESSAGE);
        assert!(matches!(err, PathVolumeSpaceQueryError::InvalidPath));
    }

    #[test]
    fn filesystem_probe_reports_missing_path_without_error() {
        let path = unique_temp_path("filesystem", "missing");

        let probe = probe_file_system_path_blocking(&path, Some(50))
            .expect("missing path should be a valid probe result");

        assert!(!probe.exists);
        assert!(!probe.is_file);
        assert!(!probe.is_dir);
        assert_eq!(probe.direct_child_count, None);
    }

    #[test]
    fn filesystem_probe_counts_direct_children_and_vhdx_files() {
        let path = unique_temp_path("filesystem", "dir");
        fs::create_dir_all(&path).expect("temp directory should be created");
        fs::write(path.join("ext4.vhdx"), b"vhdx").expect("vhdx should be created");
        fs::write(path.join("rootfs.tar"), b"tar").expect("tar should be created");

        let probe =
            probe_file_system_path_blocking(&path, Some(50)).expect("directory should be probed");

        fs::remove_dir_all(&path).expect("temp directory should be removed");

        assert!(probe.exists);
        assert!(probe.is_dir);
        assert_eq!(probe.direct_child_count, Some(2));
        assert_eq!(probe.direct_vhdx_file_count, Some(1));
        assert_eq!(probe.has_direct_children, Some(true));
        assert!(!probe.child_count_limit_exceeded);
    }

    #[test]
    fn filesystem_probe_reports_file_without_directory_fields() {
        let path = unique_temp_path("filesystem", "file").with_extension("txt");
        fs::write(&path, b"plain file").expect("temp file should be created");

        let probe =
            probe_file_system_path_blocking(&path, Some(50)).expect("file path should be probed");

        fs::remove_file(&path).expect("temp file should be removed");

        assert!(probe.exists);
        assert!(probe.is_file);
        assert!(!probe.is_dir);
        assert_eq!(probe.direct_child_count, None);
        assert_eq!(probe.direct_vhdx_file_count, None);
        assert_eq!(probe.has_direct_children, None);
    }

    #[test]
    fn filesystem_probe_stops_at_child_limit() {
        let path = unique_temp_path("filesystem", "limit");
        fs::create_dir_all(&path).expect("temp directory should be created");
        fs::write(path.join("a.vhdx"), b"vhdx").expect("vhdx should be created");
        fs::write(path.join("b.vhdx"), b"vhdx").expect("vhdx should be created");

        let probe =
            probe_file_system_path_blocking(&path, Some(1)).expect("directory should be probed");

        fs::remove_dir_all(&path).expect("temp directory should be removed");

        assert_eq!(probe.direct_child_count, Some(1));
        assert_eq!(probe.direct_vhdx_file_count, Some(1));
        assert!(probe.child_count_limit_exceeded);
        assert_eq!(probe.has_direct_children, Some(true));
    }

    #[test]
    fn filesystem_probe_errors_use_stable_user_message() {
        let err = probe_file_system_path_blocking(Path::new(""), Some(50))
            .expect_err("empty path should fail");

        assert_eq!(err.to_user_message(), FILE_SYSTEM_PROBE_ERROR_MESSAGE);
        assert!(matches!(err, FileSystemProbeError::InvalidPath));
    }

    #[test]
    fn file_size_reader_returns_file_length() {
        let path = unique_temp_path("file-size", "reader").with_extension("bin");

        fs::write(&path, b"123456789").expect("temp file should be created");

        let size = probe_file_size_blocking(&path).expect("file size should be read");

        fs::remove_file(&path).expect("temp file should be removed");

        assert_eq!(size, 9);
    }

    #[test]
    fn file_size_errors_use_stable_user_message() {
        let path = std::env::temp_dir().join("eawsl-missing-file-size.bin");
        let err = probe_file_size_blocking(&path).expect_err("missing file should fail");

        assert_eq!(err.to_user_message(), FILE_SIZE_ERROR_MESSAGE);
        assert!(matches!(err, FileSizeProbeError::Metadata { .. }));
    }

    #[test]
    fn file_size_reader_rejects_directories() {
        let path = unique_temp_path("file-size", "directory");
        fs::create_dir_all(&path).expect("temp directory should be created");

        let err = probe_file_size_blocking(&path).expect_err("directory should fail");

        fs::remove_dir_all(&path).expect("temp directory should be removed");

        assert_eq!(err.to_user_message(), FILE_SIZE_ERROR_MESSAGE);
        assert!(matches!(err, FileSizeProbeError::NotAFile(_)));
    }

    #[test]
    fn copy_file_exclusive_copies_and_reports_progress_after_start() {
        let root = unique_temp_path("filesystem-copy", "copy");
        let source = root.join("source.bin");
        let target = root.join("nested").join("target.bin");
        fs::create_dir_all(&root).expect("test root should be created");
        fs::write(&source, b"abcdef").expect("source should be written");
        let events = RefCell::new(Vec::new());

        copy_file_exclusive_with_progress(
            &source,
            &target,
            || {
                events.borrow_mut().push("started".to_string());
                Ok::<_, io::Error>(())
            },
            |percent| {
                events.borrow_mut().push(format!("{percent:.1}"));
                Ok::<_, io::Error>(())
            },
        )
        .expect("copy should succeed");

        assert_eq!(
            fs::read(&target).expect("target should be readable"),
            b"abcdef"
        );
        assert_eq!(
            events.into_inner(),
            vec!["started".to_string(), "100.0".to_string()]
        );

        fs::remove_dir_all(&root).expect("test root should be removed");
    }

    #[test]
    fn copy_file_exclusive_rejects_existing_target() {
        let root = unique_temp_path("filesystem-copy", "existing-target");
        let source = root.join("source.bin");
        let target = root.join("target.bin");
        fs::create_dir_all(&root).expect("test root should be created");
        fs::write(&source, b"new").expect("source should be written");
        fs::write(&target, b"existing").expect("target should be written");

        let err = copy_file_exclusive_with_progress(
            &source,
            &target,
            || Ok::<_, io::Error>(()),
            |_| Ok::<_, io::Error>(()),
        )
        .expect_err("existing target should fail");

        assert!(matches!(err, FileCopyError::Io(_)));
        assert_eq!(
            fs::read(&target).expect("target should remain readable"),
            b"existing"
        );

        fs::remove_dir_all(&root).expect("test root should be removed");
    }

    #[test]
    fn copy_file_exclusive_removes_target_when_progress_callback_fails() {
        let root = unique_temp_path("filesystem-copy", "progress-failure");
        let source = root.join("source.bin");
        let target = root.join("nested").join("target.bin");
        fs::create_dir_all(&root).expect("test root should be created");
        fs::write(&source, b"abcdef").expect("source should be written");

        let err = copy_file_exclusive_with_progress(&source, &target, || Ok(()), |_| Err("stop"))
            .expect_err("progress failure should fail copy");

        assert!(matches!(err, FileCopyError::Progress("stop")));
        assert!(!target.exists());
        assert_eq!(fs::read(&source).expect("source should remain"), b"abcdef");

        fs::remove_dir_all(&root).expect("test root should be removed");
    }

    #[test]
    fn write_text_creating_parent_creates_parent_directory() {
        let root = unique_temp_path("filesystem-write", "write-text");
        let path = root.join("nested").join("settings.json");

        write_text_creating_parent(&path, "{\"ok\":true}").expect("text should be written");

        assert_eq!(
            fs::read_to_string(&path).expect("text should be readable"),
            "{\"ok\":true}"
        );

        fs::remove_dir_all(&root).expect("test root should be removed");
    }
}
