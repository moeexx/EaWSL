import { get, writable } from "svelte/store";

import { getFileSize } from "$lib/tauri/system";
import type { DistroInfo } from "$lib/tauri/wsl";
import { logProbeRead, logProbeWrite } from "$lib/shared/frontend-logs";
import { normalizeOptionalText } from "$lib/shared/format";
import { toErrorMessage } from "$lib/shared/runtime";
import { joinWindowsPath } from "$lib/shared/windows-path";

export type VhdSizeStatus = "idle" | "loading" | "ready" | "missing" | "error";

export interface VhdSizeEntry {
  status: VhdSizeStatus;
  path: string | null;
  bytes: number | null;
  errorMessage: string | null;
}

export type VhdSizeCacheState = Record<string, VhdSizeEntry>;

const vhdSizeStore = writable<VhdSizeCacheState>({});
let vhdSizeRevision = 0;

export const vhdSizeCache = {
  subscribe: vhdSizeStore.subscribe,
};

export function buildDistroVhdPath(distro: DistroInfo): string | null {
  const basePath = normalizeOptionalText(distro.base_path, {
    treatUnknownAsMissing: true,
  });
  const vhdFileName = normalizeOptionalText(distro.vhd_file_name, {
    treatUnknownAsMissing: true,
  });

  return joinWindowsPath(basePath, vhdFileName);
}

export function getVhdSizeEntry(
  state: VhdSizeCacheState,
  distro: DistroInfo,
): VhdSizeEntry {
  return state[distro.name] ?? createInitialVhdSizeEntry(distro);
}

export function probeVhdSize(distro: DistroInfo): void {
  const path = buildDistroVhdPath(distro);
  const logTags = getVhdSizeLogTags(distro.name);
  const currentEntry = get(vhdSizeStore)[distro.name];

  if (currentEntry && currentEntry.path === path) {
    if (currentEntry.status === "loading") {
      return;
    }

    if (currentEntry.status !== "idle") {
      logProbeRead("VHD size", logTags, "Read cache");
      return;
    }
  }

  if (!path) {
    writeVhdSizeEntry(distro.name, {
      status: "missing",
      path: null,
      bytes: null,
      errorMessage: null,
    });
    logProbeWrite("VHD size", logTags, "Write cache");
    return;
  }

  const revision = vhdSizeRevision;

  writeVhdSizeEntry(distro.name, {
    status: "loading",
    path,
    bytes: null,
    errorMessage: null,
  });

  void getFileSize(path)
    .then((bytes): VhdSizeEntry => ({
      status: "ready",
      path,
      bytes,
      errorMessage: null,
    }))
    .catch((error): VhdSizeEntry => ({
      status: "error",
      path,
      bytes: null,
      errorMessage: toErrorMessage(error),
    }))
    .then((entry) => {
      if (revision !== vhdSizeRevision) {
        return;
      }

      let wroteEntry = false;

      vhdSizeStore.update((state) => {
        const current = state[distro.name];

        if (!current || current.path !== path || current.status !== "loading") {
          return state;
        }

        wroteEntry = true;
        return {
          ...state,
          [distro.name]: entry,
        };
      });

      if (wroteEntry) {
        logProbeWrite("VHD size", logTags, "Write cache");
      }
    });
}

export function clearVhdSizeCache(): void {
  vhdSizeRevision += 1;

  if (Object.keys(get(vhdSizeStore)).length === 0) {
    return;
  }

  vhdSizeStore.set({});
  logProbeWrite("VHD size", ["Distro list"], "Write cache");
}

function createInitialVhdSizeEntry(distro: DistroInfo): VhdSizeEntry {
  const path = buildDistroVhdPath(distro);

  return path
    ? { status: "idle", path, bytes: null, errorMessage: null }
    : { status: "missing", path: null, bytes: null, errorMessage: null };
}

function writeVhdSizeEntry(distroName: string, entry: VhdSizeEntry): void {
  vhdSizeStore.update((state) => ({
    ...state,
    [distroName]: entry,
  }));
}

function getVhdSizeLogTags(distroName: string): string[] {
  return ["Core workspace", distroName, "Expand"];
}
