use serde::{Deserialize, Serialize};

/// Phase for long-running WSL commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProgressPhase {
    Downloading,
    Installing,
    Exporting,
    Importing,
}

/// Status fallback when percentage progress is unavailable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProgressState {
    Started,
    Running,
    Completed,
}

/// Long-task progress payload.
///
/// `install` and `import` are percent-driven; `export` uses status events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProgressValue {
    Percent(f32),
    Status(ProgressState),
}

/// Progress event emitted to the outer layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub phase: ProgressPhase,
    pub value: ProgressValue,
}

pub(crate) fn percent_event(phase: ProgressPhase, percent: f32) -> ProgressEvent {
    ProgressEvent {
        phase,
        value: ProgressValue::Percent(percent),
    }
}

pub(crate) fn status_event(phase: ProgressPhase, state: ProgressState) -> ProgressEvent {
    ProgressEvent {
        phase,
        value: ProgressValue::Status(state),
    }
}
