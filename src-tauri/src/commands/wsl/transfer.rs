use tauri::AppHandle;

use crate::bridge::progress::{run_with_progress, TRANSFER_PROGRESS_EVENT};
use crate::commands::shared::dto::{
    ExportDistroRequest, ImportDistroInPlaceRequest, ImportDistroRequest, InstallDistroRequest,
};
use crate::commands::shared::error::map_command_error;

use super::vhdx_relocation::prepare_vhdx_relocation;

#[tauri::command]
pub async fn install_distro(app: AppHandle, req: InstallDistroRequest) -> Result<(), String> {
    let InstallDistroRequest {
        request_id,
        distro,
        options,
    } = req;
    let options = options.into_core().map_err(map_command_error)?;

    run_with_progress(
        app,
        TRANSFER_PROGRESS_EVENT,
        request_id,
        distro.clone(),
        move |tx, cancel_token| async move {
            wsl_core::install_distro(&distro, options, tx, cancel_token).await
        },
    )
    .await
}

#[tauri::command]
pub async fn import_distro(app: AppHandle, req: ImportDistroRequest) -> Result<(), String> {
    let ImportDistroRequest {
        request_id,
        distro,
        location,
        file,
    } = req;

    run_with_progress(
        app,
        TRANSFER_PROGRESS_EVENT,
        request_id,
        distro.clone(),
        move |tx, cancel_token| async move {
            wsl_core::import_distro(&distro, &location, &file, tx, cancel_token).await
        },
    )
    .await
}

#[tauri::command]
pub async fn import_distro_in_place(
    app: AppHandle,
    req: ImportDistroInPlaceRequest,
) -> Result<(), String> {
    let ImportDistroInPlaceRequest {
        request_id,
        distro,
        source_vhdx,
        target_directory,
    } = req;
    let relocation = prepare_vhdx_relocation(
        app.clone(),
        TRANSFER_PROGRESS_EVENT,
        &request_id,
        &distro,
        source_vhdx,
        target_directory,
    )
    .await?;
    let final_vhdx = relocation.final_vhdx.clone();

    let import_result = run_with_progress(
        app,
        TRANSFER_PROGRESS_EVENT,
        request_id,
        distro.clone(),
        move |tx, cancel_token| async move {
            wsl_core::import_distro_in_place(&distro, &final_vhdx, tx, cancel_token).await
        },
    )
    .await;

    if let Err(err) = import_result {
        relocation.cleanup_failed_import();
        return Err(err);
    }

    Ok(())
}

#[tauri::command]
pub async fn export_distro(app: AppHandle, req: ExportDistroRequest) -> Result<(), String> {
    let ExportDistroRequest {
        request_id,
        distro,
        file,
        format,
    } = req;

    run_with_progress(
        app,
        TRANSFER_PROGRESS_EVENT,
        request_id,
        distro.clone(),
        move |tx, cancel_token| async move {
            wsl_core::export_distro(&distro, &file, format, tx, cancel_token).await
        },
    )
    .await
}
