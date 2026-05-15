use std::future::Future;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use wsl_core::{
    ProgressEvent as CoreProgressEvent, ProgressPhase as CoreProgressPhase, ProgressState,
    ProgressValue as CoreProgressValue, WslError,
};

use crate::commands::shared::error::map_command_error;

pub(crate) const TRANSFER_PROGRESS_EVENT: &str = "distro:transfer-progress";
const PROGRESS_CHANNEL_CAPACITY: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferProgressPhase {
    Downloading,
    Installing,
    Exporting,
    Importing,
    Copying,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferProgressValue {
    Percent(f32),
    Status(ProgressState),
}

/// Tauri payload that relays `wsl-core` progress.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransferProgressEvent {
    pub phase: TransferProgressPhase,
    pub value: TransferProgressValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DistroProgressEvent {
    pub request_id: String,
    pub distro: String,
    pub progress: TransferProgressEvent,
}

pub(crate) trait ProgressEmitter: Clone + Send + Sync + 'static {
    fn emit_progress(&self, event_name: &str, payload: DistroProgressEvent) -> Result<(), String>;
}

impl<R> ProgressEmitter for AppHandle<R>
where
    R: tauri::Runtime,
{
    fn emit_progress(&self, event_name: &str, payload: DistroProgressEvent) -> Result<(), String> {
        self.emit(event_name, payload)
            .map_err(|err| err.to_string())
    }
}

async fn relay_progress<E>(
    emitter: E,
    event_name: &'static str,
    request_id: String,
    distro: String,
    mut rx: mpsc::Receiver<CoreProgressEvent>,
) -> Result<(), String>
where
    E: ProgressEmitter,
{
    while let Some(progress) = rx.recv().await {
        emit_transfer_progress(
            &emitter,
            event_name,
            &request_id,
            &distro,
            map_core_progress(progress),
        )?;
    }

    Ok(())
}

pub(crate) fn copy_progress_event(percent: f32) -> TransferProgressEvent {
    TransferProgressEvent {
        phase: TransferProgressPhase::Copying,
        value: TransferProgressValue::Percent(percent),
    }
}

pub(crate) fn status_progress_event(
    phase: TransferProgressPhase,
    state: ProgressState,
) -> TransferProgressEvent {
    TransferProgressEvent {
        phase,
        value: TransferProgressValue::Status(state),
    }
}

pub(crate) fn emit_transfer_progress<E>(
    emitter: &E,
    event_name: &str,
    request_id: &str,
    distro: &str,
    progress: TransferProgressEvent,
) -> Result<(), String>
where
    E: ProgressEmitter,
{
    emitter.emit_progress(
        event_name,
        DistroProgressEvent {
            request_id: request_id.to_string(),
            distro: distro.to_string(),
            progress,
        },
    )
}

fn map_core_progress(progress: CoreProgressEvent) -> TransferProgressEvent {
    TransferProgressEvent {
        phase: match progress.phase {
            CoreProgressPhase::Downloading => TransferProgressPhase::Downloading,
            CoreProgressPhase::Installing => TransferProgressPhase::Installing,
            CoreProgressPhase::Exporting => TransferProgressPhase::Exporting,
            CoreProgressPhase::Importing => TransferProgressPhase::Importing,
        },
        value: match progress.value {
            CoreProgressValue::Percent(percent) => TransferProgressValue::Percent(percent),
            CoreProgressValue::Status(state) => TransferProgressValue::Status(state),
        },
    }
}

/// Run a long task and relay core progress as Tauri events.
pub(crate) async fn run_with_progress<E, Op, Fut, T>(
    emitter: E,
    event_name: &'static str,
    request_id: String,
    distro: String,
    operation: Op,
) -> Result<T, String>
where
    E: ProgressEmitter,
    Op: FnOnce(mpsc::Sender<CoreProgressEvent>, CancellationToken) -> Fut,
    Fut: Future<Output = Result<T, WslError>>,
{
    let (tx, rx) = mpsc::channel(PROGRESS_CHANNEL_CAPACITY);
    let relay_task = tokio::spawn(relay_progress(emitter, event_name, request_id, distro, rx));

    let result = operation(tx, CancellationToken::new())
        .await
        .map_err(map_command_error);

    let relay_result = relay_task.await.map_err(|err| err.to_string())?;
    let value = result?;
    relay_result?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tokio::sync::mpsc;
    use wsl_core::{
        ProgressEvent, ProgressPhase, ProgressState, ProgressValue, WslCommandContext, WslError,
    };

    use super::{
        run_with_progress, DistroProgressEvent, ProgressEmitter, TransferProgressEvent,
        TransferProgressPhase, TransferProgressValue, TRANSFER_PROGRESS_EVENT,
    };

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

    #[derive(Clone)]
    struct FailingEmitter;

    impl ProgressEmitter for FailingEmitter {
        fn emit_progress(
            &self,
            _event_name: &str,
            _payload: DistroProgressEvent,
        ) -> Result<(), String> {
            Err("emit failed".to_string())
        }
    }

    #[tokio::test]
    async fn run_with_progress_wraps_and_emits_events() {
        let emitter = RecordingEmitter::default();

        run_with_progress(
            emitter.clone(),
            TRANSFER_PROGRESS_EVENT,
            "req-1".to_string(),
            "Ubuntu".to_string(),
            move |tx, _cancel_token| async move {
                tx.send(wsl_core::ProgressEvent {
                    phase: ProgressPhase::Installing,
                    value: ProgressValue::Status(ProgressState::Running),
                })
                .await
                .expect("send progress event");
                Ok(())
            },
        )
        .await
        .expect("progress bridge should succeed");

        assert_eq!(
            emitter.events(),
            vec![(
                TRANSFER_PROGRESS_EVENT.to_string(),
                DistroProgressEvent {
                    request_id: "req-1".to_string(),
                    distro: "Ubuntu".to_string(),
                    progress: TransferProgressEvent {
                        phase: TransferProgressPhase::Installing,
                        value: TransferProgressValue::Status(ProgressState::Running),
                    },
                },
            )]
        );
    }

    #[tokio::test]
    async fn run_with_progress_maps_core_progress_phases() {
        let emitter = RecordingEmitter::default();

        run_with_progress(
            emitter.clone(),
            TRANSFER_PROGRESS_EVENT,
            "req-1".to_string(),
            "Ubuntu".to_string(),
            move |tx, _cancel_token| async move {
                for progress in [
                    ProgressEvent {
                        phase: ProgressPhase::Downloading,
                        value: ProgressValue::Percent(42.5),
                    },
                    ProgressEvent {
                        phase: ProgressPhase::Exporting,
                        value: ProgressValue::Status(ProgressState::Started),
                    },
                    ProgressEvent {
                        phase: ProgressPhase::Importing,
                        value: ProgressValue::Status(ProgressState::Completed),
                    },
                ] {
                    tx.send(progress).await.expect("send progress event");
                }
                Ok(())
            },
        )
        .await
        .expect("progress bridge should succeed");

        let events = emitter
            .events()
            .into_iter()
            .map(|(_, payload)| payload.progress)
            .collect::<Vec<_>>();
        assert_eq!(
            events,
            vec![
                TransferProgressEvent {
                    phase: TransferProgressPhase::Downloading,
                    value: TransferProgressValue::Percent(42.5),
                },
                TransferProgressEvent {
                    phase: TransferProgressPhase::Exporting,
                    value: TransferProgressValue::Status(ProgressState::Started),
                },
                TransferProgressEvent {
                    phase: TransferProgressPhase::Importing,
                    value: TransferProgressValue::Status(ProgressState::Completed),
                },
            ]
        );
    }

    #[tokio::test]
    async fn run_with_progress_returns_user_facing_errors() {
        let emitter = RecordingEmitter::default();

        let err = run_with_progress(
            emitter,
            TRANSFER_PROGRESS_EVENT,
            "req-2".to_string(),
            "Ubuntu".to_string(),
            move |_tx: mpsc::Sender<ProgressEvent>, _cancel_token| async move {
                Err::<(), WslError>(WslError::InvalidArgument {
                    context: WslCommandContext::Install,
                    raw_output: "bad args".to_string(),
                })
            },
        )
        .await
        .expect_err("operation failure should bubble up");

        assert_eq!(err, "The WSL command arguments are invalid.");
    }

    #[tokio::test]
    async fn run_with_progress_returns_emitter_errors() {
        let err = run_with_progress(
            FailingEmitter,
            TRANSFER_PROGRESS_EVENT,
            "req-3".to_string(),
            "Ubuntu".to_string(),
            move |tx, _cancel_token| async move {
                tx.send(ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Percent(100.0),
                })
                .await
                .expect("send progress event");
                Ok(())
            },
        )
        .await
        .expect_err("emitter failure should bubble up");

        assert_eq!(err, "emit failed");
    }
}
