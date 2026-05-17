import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { normalizeTauriCommandError } from "./errors";

const TRANSFER_PROGRESS_EVENT = "distro:transfer-progress";

type KnownDistroState = "Running" | "Stopped" | "Installing";

type UnknownDistroState = {
  Unknown: string;
};

export type DistroState = KnownDistroState | UnknownDistroState;

export type ProgressPhase =
  | "Downloading"
  | "Installing"
  | "Exporting"
  | "Importing"
  | "Copying";

export type ProgressState = "Started" | "Running" | "Completed";

export type ProgressValue = { Percent: number } | { Status: ProgressState };

interface ProgressEvent {
  phase: ProgressPhase;
  value: ProgressValue;
}

export interface DistroProgressEvent {
  requestId: string;
  distro: string;
  progress: ProgressEvent;
}

export interface DistroInfo {
  name: string;
  state: DistroState;
  version: number;
  is_default: boolean;
  base_path: string | null;
  vhd_file_name: string | null;
  flavor: string | null;
  os_version: string | null;
  default_uid: number | null;
}

export interface WslVersion {
  wsl: string;
  kernel: string | null;
  wslg: string | null;
  msrdc: string | null;
  direct3d: string | null;
  dxcore: string | null;
  windows: string;
}

export interface OnlineDistro {
  name: string;
  friendly_name: string;
}

export interface InstallOptions {
  name: string | null;
  location: string | null;
  vhdSize: string | null;
  fixedVhd: boolean;
}

export interface InstallDistroRequest {
  requestId: string;
  distro: string;
  options: InstallOptions;
}

export interface ImportDistroRequest {
  requestId: string;
  distro: string;
  location: string;
  file: string;
}

export interface ImportDistroInPlaceRequest {
  requestId: string;
  distro: string;
  sourceVhdx: string;
  targetDirectory: string | null;
}

export type ExportFormat = "Tar" | "TarGz" | "TarXz" | "Vhd";

export interface ExportDistroRequest {
  requestId: string;
  distro: string;
  file: string;
  format: ExportFormat;
}

function mapInstallOptions(options: InstallOptions) {
  return {
    name: options.name,
    location: options.location,
    vhd_size: options.vhdSize,
    fixed_vhd: options.fixedVhd,
  };
}

async function invokeWsl<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeTauriCommandError(error);
  }
}

export async function listDistros(): Promise<DistroInfo[]> {
  return invokeWsl<DistroInfo[]>("list_distros");
}

export async function getWslVersion(): Promise<WslVersion> {
  return invokeWsl<WslVersion>("get_wsl_version");
}

export async function listOnlineDistros(): Promise<OnlineDistro[]> {
  return invokeWsl<OnlineDistro[]>("list_online_distros");
}

export async function setDefaultDistro(distro: string): Promise<void> {
  return invokeWsl("set_default_distro", { distro });
}

export async function terminateDistro(distro: string): Promise<void> {
  return invokeWsl("terminate_distro", { distro });
}

export async function shutdownWsl(force: boolean): Promise<void> {
  return invokeWsl("shutdown_wsl", { force });
}

export async function unregisterDistro(distro: string): Promise<void> {
  return invokeWsl("unregister_distro", { distro });
}

export async function installDistro(req: InstallDistroRequest): Promise<void> {
  return invokeWsl("install_distro", {
    req: {
      requestId: req.requestId,
      distro: req.distro,
      options: mapInstallOptions(req.options),
    },
  });
}

export async function launchLegacyInstallTerminal(
  distro: string,
): Promise<void> {
  return invokeWsl("launch_legacy_install_terminal", { distro });
}

export async function importDistro(req: ImportDistroRequest): Promise<void> {
  return invokeWsl("import_distro", {
    req: {
      requestId: req.requestId,
      distro: req.distro,
      location: req.location,
      file: req.file,
    },
  });
}

export async function importDistroInPlace(
  req: ImportDistroInPlaceRequest,
): Promise<void> {
  return invokeWsl("import_distro_in_place", {
    req: {
      requestId: req.requestId,
      distro: req.distro,
      sourceVhdx: req.sourceVhdx,
      targetDirectory: req.targetDirectory,
    },
  });
}

export async function exportDistro(req: ExportDistroRequest): Promise<void> {
  return invokeWsl("export_distro", {
    req: {
      requestId: req.requestId,
      distro: req.distro,
      file: req.file,
      format: req.format,
    },
  });
}

export async function listenTransferProgress(
  handler: (payload: DistroProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<DistroProgressEvent>(TRANSFER_PROGRESS_EVENT, (event) => {
    handler(event.payload);
  });
}
