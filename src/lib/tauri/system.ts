import { invoke } from "@tauri-apps/api/core";

import { normalizeTauriCommandError } from "./errors";

export interface WindowsOverview {
  productName: string | null;
  displayVersion: string | null;
  buildNumber: string | null;
}

export interface CpuOverview {
  model: string | null;
  maxClockMhz: number | null;
  coreCount: number | null;
  logicalProcessorCount: number | null;
}

export interface MemoryOverview {
  totalBytes: number | null;
  speedMts: number | null;
  usedSlots: number | null;
  totalSlots: number | null;
}

export interface GpuOverview {
  name: string | null;
  memoryBytes: number | null;
  driverVersion: string | null;
}

export interface StorageOverview {
  totalBytes: number | null;
  usedBytes: number | null;
  freeBytes: number | null;
  volumeCount: number | null;
}

export interface SystemOverview {
  windows: WindowsOverview;
  cpu: CpuOverview;
  memory: MemoryOverview;
  gpu: GpuOverview | null;
  storage: StorageOverview;
}

export type SystemOverviewScope = "full" | "storage";

export interface GetSystemOverviewOptions {
  scope?: SystemOverviewScope;
}

export interface PathVolumeSpace {
  volumeRoot: string;
  freeBytes: number;
}

export interface FileSystemPathProbe {
  exists: boolean;
  isFile: boolean;
  isDir: boolean;
  directChildCount: number | null;
  childCountLimitExceeded: boolean;
  directVhdxFileCount: number | null;
  hasDirectChildren: boolean | null;
}

async function invokeSystem<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeTauriCommandError(error);
  }
}

export async function getFileSize(path: string): Promise<number> {
  return invokeSystem<number>("get_file_size", { path });
}

export async function getPathVolumeSpace(
  path: string,
): Promise<PathVolumeSpace> {
  return invokeSystem<PathVolumeSpace>("get_path_volume_space", { path });
}

export async function probeFileSystemPath(
  path: string,
  childLimit: number | null = null,
): Promise<FileSystemPathProbe> {
  return invokeSystem<FileSystemPathProbe>("probe_file_system_path", {
    path,
    childLimit,
  });
}

export async function getSystemOverview(
  options: GetSystemOverviewOptions = {},
): Promise<SystemOverview> {
  return invokeSystem<SystemOverview>(
    "get_system_overview",
    options.scope ? { scope: options.scope } : undefined,
  );
}
