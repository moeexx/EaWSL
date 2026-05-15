use crate::domain::model::progress::{
    percent_event, status_event, ProgressEvent, ProgressPhase, ProgressState,
};
use crate::infrastructure::wsl_cli::decoder::StreamingTextRecords;
use regex::Regex;
use std::sync::OnceLock;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) trait CommandProgressHandler: Send {
    fn on_started(&mut self);
    fn on_stdout_chunk(&mut self, chunk: &[u8]);
    fn on_stderr_chunk(&mut self, _chunk: &[u8]) {}
    fn finish(&mut self) {}
    fn on_success(&mut self);
}

pub(crate) struct InstallProgressHandler {
    event_tx: UnboundedSender<ProgressEvent>,
    stdout_records: StreamingTextRecords,
    phase: ProgressPhase,
    last_percent: Option<f32>,
}

impl InstallProgressHandler {
    pub(crate) fn new(event_tx: UnboundedSender<ProgressEvent>) -> Self {
        Self {
            event_tx,
            stdout_records: StreamingTextRecords::new(),
            phase: ProgressPhase::Downloading,
            last_percent: None,
        }
    }

    pub(crate) fn on_started(&mut self) {
        let _ = self.event_tx.send(status_event(
            ProgressPhase::Downloading,
            ProgressState::Started,
        ));
    }

    pub(crate) fn on_stdout_chunk(&mut self, chunk: &[u8]) {
        for record in self.stdout_records.push_bytes(chunk) {
            self.handle_record(&record);
        }
    }

    pub(crate) fn finish(&mut self) {
        for record in self.stdout_records.finish() {
            self.handle_record(&record);
        }
    }

    pub(crate) fn on_success(&mut self) {
        let _ = self
            .event_tx
            .send(status_event(self.phase.clone(), ProgressState::Completed));
    }

    fn handle_record(&mut self, record: &str) {
        if let Some(percent) = extract_progress_percent(record) {
            if self.last_percent.is_some_and(|last| percent < last) {
                self.phase = ProgressPhase::Installing;
            }

            self.last_percent = Some(percent);
            let _ = self
                .event_tx
                .send(percent_event(self.phase.clone(), percent));
        }
    }
}

impl CommandProgressHandler for InstallProgressHandler {
    fn on_started(&mut self) {
        Self::on_started(self);
    }

    fn on_stdout_chunk(&mut self, chunk: &[u8]) {
        Self::on_stdout_chunk(self, chunk);
    }

    fn finish(&mut self) {
        Self::finish(self);
    }

    fn on_success(&mut self) {
        Self::on_success(self);
    }
}

pub(crate) struct ImportProgressHandler {
    event_tx: UnboundedSender<ProgressEvent>,
    stdout_records: StreamingTextRecords,
}

impl ImportProgressHandler {
    pub(crate) fn new(event_tx: UnboundedSender<ProgressEvent>) -> Self {
        Self {
            event_tx,
            stdout_records: StreamingTextRecords::new(),
        }
    }

    pub(crate) fn on_started(&mut self) {
        let _ = self.event_tx.send(status_event(
            ProgressPhase::Importing,
            ProgressState::Started,
        ));
    }

    pub(crate) fn on_stdout_chunk(&mut self, chunk: &[u8]) {
        for record in self.stdout_records.push_bytes(chunk) {
            self.handle_record(&record);
        }
    }

    pub(crate) fn finish(&mut self) {
        for record in self.stdout_records.finish() {
            self.handle_record(&record);
        }
    }

    pub(crate) fn on_success(&mut self) {
        let _ = self.event_tx.send(status_event(
            ProgressPhase::Importing,
            ProgressState::Completed,
        ));
    }

    fn handle_record(&mut self, record: &str) {
        if let Some(percent) = extract_progress_percent(record) {
            let _ = self
                .event_tx
                .send(percent_event(ProgressPhase::Importing, percent));
        }
    }
}

impl CommandProgressHandler for ImportProgressHandler {
    fn on_started(&mut self) {
        Self::on_started(self);
    }

    fn on_stdout_chunk(&mut self, chunk: &[u8]) {
        Self::on_stdout_chunk(self, chunk);
    }

    fn finish(&mut self) {
        Self::finish(self);
    }

    fn on_success(&mut self) {
        Self::on_success(self);
    }
}

pub(crate) struct ExportProgressHandler {
    event_tx: UnboundedSender<ProgressEvent>,
    running_emitted: bool,
}

impl ExportProgressHandler {
    pub(crate) fn new(event_tx: UnboundedSender<ProgressEvent>) -> Self {
        Self {
            event_tx,
            running_emitted: false,
        }
    }

    pub(crate) fn on_started(&mut self) {
        let _ = self.event_tx.send(status_event(
            ProgressPhase::Exporting,
            ProgressState::Started,
        ));
    }

    pub(crate) fn on_output_chunk(&mut self, chunk: &[u8]) {
        if self.running_emitted || chunk.is_empty() {
            return;
        }

        self.running_emitted = true;
        let _ = self.event_tx.send(status_event(
            ProgressPhase::Exporting,
            ProgressState::Running,
        ));
    }

    pub(crate) fn on_success(&mut self) {
        let _ = self.event_tx.send(status_event(
            ProgressPhase::Exporting,
            ProgressState::Completed,
        ));
    }
}

impl CommandProgressHandler for ExportProgressHandler {
    fn on_started(&mut self) {
        Self::on_started(self);
    }

    fn on_stdout_chunk(&mut self, chunk: &[u8]) {
        Self::on_output_chunk(self, chunk);
    }

    fn on_stderr_chunk(&mut self, chunk: &[u8]) {
        Self::on_output_chunk(self, chunk);
    }

    fn on_success(&mut self) {
        Self::on_success(self);
    }
}

fn extract_progress_percent(output: &str) -> Option<f32> {
    progress_percent_regex()
        .captures_iter(output)
        .last()
        .and_then(|captures| captures.get(1))
        .and_then(|matched| matched.as_str().parse::<f32>().ok())
}

fn progress_percent_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(\d+\.?\d*)\s*%").expect("valid progress percent regex"))
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::{
        CommandProgressHandler, ExportProgressHandler, ImportProgressHandler,
        InstallProgressHandler,
    };
    use crate::{ProgressEvent, ProgressPhase, ProgressState, ProgressValue};

    fn drain_events(rx: &mut mpsc::UnboundedReceiver<ProgressEvent>) -> Vec<ProgressEvent> {
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
        events
    }

    #[test]
    fn install_progress_switches_to_installing_when_percent_drops() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut handler = InstallProgressHandler::new(tx);

        handler.on_started();
        handler.on_stdout_chunk(b"[================95.0%]\r");
        handler.on_stdout_chunk(b"[==5.0%]\r");
        handler.on_success();

        assert_eq!(
            drain_events(&mut rx),
            vec![
                ProgressEvent {
                    phase: ProgressPhase::Downloading,
                    value: ProgressValue::Status(ProgressState::Started),
                },
                ProgressEvent {
                    phase: ProgressPhase::Downloading,
                    value: ProgressValue::Percent(95.0),
                },
                ProgressEvent {
                    phase: ProgressPhase::Installing,
                    value: ProgressValue::Percent(5.0),
                },
                ProgressEvent {
                    phase: ProgressPhase::Installing,
                    value: ProgressValue::Status(ProgressState::Completed),
                },
            ]
        );
    }

    #[test]
    fn import_progress_flushes_pending_percent_on_finish() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut handler = ImportProgressHandler::new(tx);

        handler.on_started();
        handler.on_stdout_chunk(b"[===========20.4%");
        handler.finish();
        handler.on_success();

        assert_eq!(
            drain_events(&mut rx),
            vec![
                ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Status(ProgressState::Started),
                },
                ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Percent(20.4),
                },
                ProgressEvent {
                    phase: ProgressPhase::Importing,
                    value: ProgressValue::Status(ProgressState::Completed),
                },
            ]
        );
    }

    #[test]
    fn export_progress_emits_running_once_for_stdout_and_stderr() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut handler = ExportProgressHandler::new(tx);

        handler.on_started();
        handler.on_stdout_chunk(b"exporting");
        handler.on_stderr_chunk(b"still exporting");
        handler.on_success();

        assert_eq!(
            drain_events(&mut rx),
            vec![
                ProgressEvent {
                    phase: ProgressPhase::Exporting,
                    value: ProgressValue::Status(ProgressState::Started),
                },
                ProgressEvent {
                    phase: ProgressPhase::Exporting,
                    value: ProgressValue::Status(ProgressState::Running),
                },
                ProgressEvent {
                    phase: ProgressPhase::Exporting,
                    value: ProgressValue::Status(ProgressState::Completed),
                },
            ]
        );
    }
}
