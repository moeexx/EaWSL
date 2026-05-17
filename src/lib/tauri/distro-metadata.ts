import { invoke } from "@tauri-apps/api/core";

import { normalizeTauriCommandError } from "./errors";

export interface DistroMetadata {
  name: string;
  friendlyName: string;
  amd64Url: string | null;
  arm64Url: string | null;
  modern: boolean;
}

async function invokeDistroMetadata<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeTauriCommandError(error);
  }
}

export function getDistroMetadata(): Promise<
  DistroMetadata[]
> {
  return invokeDistroMetadata("get_distro_metadata");
}

export function refreshDistroMetadata(): Promise<
  DistroMetadata[]
> {
  return invokeDistroMetadata("refresh_distro_metadata");
}

