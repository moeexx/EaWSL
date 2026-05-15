use std::future::Future;
use std::path::Path;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::application::port::progress_sink::ProgressSink;
use crate::application::port::wsl_cli::{
    WslLifecyclePort, WslQueryPort, WslStoragePort, WslTransferPort,
};
use crate::domain::model::distro::InstalledDistroSnapshot;
use crate::{
    DiskSize, ExportFormat, InstallOptions, OnlineDistro, ProgressEvent, WslError, WslVersion,
};

const DEFAULT_PROGRESS_CHANNEL_CAPACITY: usize = 32;

pub(crate) struct SystemWslCliAdapter;

impl SystemWslCliAdapter {
    async fn forward_progress<F, S>(
        &self,
        sink: &S,
        producer: F,
        mut rx: mpsc::Receiver<ProgressEvent>,
    ) -> Result<(), WslError>
    where
        F: Future<Output = Result<(), WslError>>,
        S: ProgressSink + ?Sized,
    {
        tokio::pin!(producer);

        let result = loop {
            tokio::select! {
                result = &mut producer => break result,
                maybe_event = rx.recv() => {
                    if let Some(event) = maybe_event {
                        sink.emit(event).await;
                    }
                }
            }
        };

        while let Some(event) = rx.recv().await {
            sink.emit(event).await;
        }

        result
    }
}

#[allow(async_fn_in_trait)]
impl WslQueryPort for SystemWslCliAdapter {
    async fn get_wsl_version(&self) -> Result<WslVersion, WslError> {
        super::query::get_wsl_version().await
    }

    async fn list_installed_distros(&self) -> Result<Vec<InstalledDistroSnapshot>, WslError> {
        super::query::list_installed_distros().await
    }

    async fn list_online_distros(&self) -> Result<Vec<OnlineDistro>, WslError> {
        super::query::list_online_distros().await
    }
}

#[allow(async_fn_in_trait)]
impl WslLifecyclePort for SystemWslCliAdapter {
    async fn set_default_distro(&self, distro: &str) -> Result<(), WslError> {
        super::lifecycle::set_default_distro(distro).await
    }

    async fn terminate_distro(&self, distro: &str) -> Result<(), WslError> {
        super::lifecycle::terminate_distro(distro).await
    }

    async fn shutdown_wsl(&self, force: bool) -> Result<(), WslError> {
        super::lifecycle::shutdown_wsl(force).await
    }

    async fn unregister_distro(&self, distro: &str) -> Result<(), WslError> {
        super::lifecycle::unregister_distro(distro).await
    }
}

#[allow(async_fn_in_trait)]
impl WslStoragePort for SystemWslCliAdapter {
    async fn move_distro(&self, distro: &str, new_location: &Path) -> Result<(), WslError> {
        super::storage::move_distro(distro, new_location).await
    }

    async fn resize_distro(&self, distro: &str, size: DiskSize) -> Result<(), WslError> {
        super::storage::resize_distro(distro, size).await
    }
}

#[allow(async_fn_in_trait)]
impl WslTransferPort for SystemWslCliAdapter {
    async fn install_distro<S>(
        &self,
        distro: &str,
        opts: InstallOptions,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<(), WslError>
    where
        S: ProgressSink + ?Sized,
    {
        let (tx, rx) = mpsc::channel(DEFAULT_PROGRESS_CHANNEL_CAPACITY);
        let producer = super::transfer::install_distro(distro, opts, tx, cancel_token);
        self.forward_progress(sink, producer, rx).await
    }

    async fn import_distro<S>(
        &self,
        distro: &str,
        location: &Path,
        file: &Path,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<(), WslError>
    where
        S: ProgressSink + ?Sized,
    {
        let (tx, rx) = mpsc::channel(DEFAULT_PROGRESS_CHANNEL_CAPACITY);
        let producer = super::transfer::import_distro(distro, location, file, tx, cancel_token);
        self.forward_progress(sink, producer, rx).await
    }

    async fn import_distro_in_place<S>(
        &self,
        distro: &str,
        vhdx: &Path,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<(), WslError>
    where
        S: ProgressSink + ?Sized,
    {
        let (tx, rx) = mpsc::channel(DEFAULT_PROGRESS_CHANNEL_CAPACITY);
        let producer = super::transfer::import_distro_in_place(distro, vhdx, tx, cancel_token);
        self.forward_progress(sink, producer, rx).await
    }

    async fn export_distro<S>(
        &self,
        distro: &str,
        file: &Path,
        format: ExportFormat,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<(), WslError>
    where
        S: ProgressSink + ?Sized,
    {
        let (tx, rx) = mpsc::channel(DEFAULT_PROGRESS_CHANNEL_CAPACITY);
        let producer = super::transfer::export_distro(distro, file, format, tx, cancel_token);
        self.forward_progress(sink, producer, rx).await
    }
}
