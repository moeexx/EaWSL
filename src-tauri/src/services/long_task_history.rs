use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::services::system::filesystem::{self, WriteTextError};

const LONG_TASK_HISTORY_FILE_NAME: &str = "long-tasks.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedLongTask {
    pub request_id: String,
    pub distro: String,
    pub operation: String,
    pub status: String,
    pub phase: Option<String>,
    pub percent: Option<f32>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub error: Option<String>,
    pub location: Option<String>,
    pub logo_src: String,
    #[serde(default)]
    pub interrupted: bool,
}

pub async fn get(app: AppHandle) -> Result<Vec<PersistedLongTask>, String> {
    let path = long_task_history_file_path(&app)?;
    read_long_task_history(path).await
}

pub async fn save(app: AppHandle, tasks: Vec<PersistedLongTask>) -> Result<(), String> {
    let path = long_task_history_file_path(&app)?;
    write_long_task_history(path, tasks).await
}

fn long_task_history_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join(LONG_TASK_HISTORY_FILE_NAME))
        .map_err(|error| format!("failed to resolve app data directory: {error}"))
}

async fn read_long_task_history(path: PathBuf) -> Result<Vec<PersistedLongTask>, String> {
    tokio::task::spawn_blocking(move || read_long_task_history_blocking(&path))
        .await
        .map_err(|error| format!("failed to join long task history read task: {error}"))
}

async fn write_long_task_history(
    path: PathBuf,
    tasks: Vec<PersistedLongTask>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || write_long_task_history_blocking(&path, &tasks))
        .await
        .map_err(|error| format!("failed to join long task history write task: {error}"))?
}

fn read_long_task_history_blocking(path: &Path) -> Vec<PersistedLongTask> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    serde_json::from_str(&raw).unwrap_or_default()
}

fn write_long_task_history_blocking(
    path: &Path,
    tasks: &[PersistedLongTask],
) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(tasks)
        .map_err(|source| format!("failed to serialize long task history: {source}"))?;
    filesystem::write_text_creating_parent(path, &raw).map_err(format_long_task_history_write_error)
}

fn format_long_task_history_write_error(error: WriteTextError) -> String {
    match error {
        WriteTextError::CreateDirectory { path, source } => {
            format!(
                "failed to create long task history directory {}: {source}",
                path.display()
            )
        }
        WriteTextError::Write { path, source } => {
            format!(
                "failed to write long task history file {}: {source}",
                path.display()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{
        read_long_task_history_blocking, write_long_task_history_blocking, PersistedLongTask,
    };
    use crate::commands::test_support::unique_temp_path;

    fn unique_test_path(name: &str) -> PathBuf {
        unique_temp_path("long-task-history", "test").join(name)
    }

    #[test]
    fn write_and_read_long_task_history_round_trips() {
        let path = unique_test_path("long-tasks.json");
        let tasks = vec![PersistedLongTask {
            request_id: "req-1".to_string(),
            distro: "Ubuntu".to_string(),
            operation: "export".to_string(),
            status: "completed".to_string(),
            phase: Some("Exporting".to_string()),
            percent: Some(100.0),
            started_at: "2026-05-16T00:00:00.000Z".to_string(),
            ended_at: Some("2026-05-16T00:01:00.000Z".to_string()),
            error: None,
            location: Some("D:/exports/ubuntu.tar".to_string()),
            logo_src: "/distro-logos/ubuntu.ico".to_string(),
            interrupted: false,
        }];

        write_long_task_history_blocking(&path, &tasks)
            .expect("long task history should be written");

        assert_eq!(read_long_task_history_blocking(&path), tasks);

        let parent = path.parent().expect("test path should have parent");
        fs::remove_dir_all(parent).expect("test directory should be removed");
    }

    #[test]
    fn read_long_task_history_falls_back_for_missing_or_damaged_file() {
        let missing = unique_test_path("missing-long-tasks.json");
        assert!(read_long_task_history_blocking(&missing).is_empty());

        let damaged = unique_test_path("damaged-long-tasks.json");
        let damaged_parent = damaged.parent().expect("test path should have parent");
        fs::create_dir_all(damaged_parent).expect("test directory should be created");
        fs::write(&damaged, "{ not json").expect("damaged history should be written");
        assert!(read_long_task_history_blocking(&damaged).is_empty());
        fs::remove_dir_all(damaged_parent).expect("test directory should be removed");
    }
}
