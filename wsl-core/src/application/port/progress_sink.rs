use tokio::sync::mpsc::Sender;

use crate::ProgressEvent;

#[allow(async_fn_in_trait)]
pub(crate) trait ProgressSink: Send + Sync {
    async fn emit(&self, event: ProgressEvent);
}

pub(crate) struct ChannelProgressSink {
    tx: Sender<ProgressEvent>,
}

impl ChannelProgressSink {
    pub(crate) fn new(tx: Sender<ProgressEvent>) -> Self {
        Self { tx }
    }
}

impl ProgressSink for ChannelProgressSink {
    async fn emit(&self, event: ProgressEvent) {
        let _ = self.tx.send(event).await;
    }
}
