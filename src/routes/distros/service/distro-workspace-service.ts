import { get, type Readable } from "svelte/store";
import { open } from "@tauri-apps/plugin-dialog";

import { openConfirmDialog, type ConfirmDialogOptions } from "$lib/feedback/confirm-dialog";
import {
  buildDistroVhdPath,
  getVhdSizeEntry,
  probeVhdSize,
  vhdSizeCache,
  type VhdSizeCacheState,
  type VhdSizeEntry,
} from "$lib/probes/distro-vhd-size";
import {
  queryCache,
  refreshDistroWorkspace,
  refreshDistrosAfterAction,
  type QueryCacheState,
  type QueryRefreshResult,
} from "$lib/query-cache";
import { pushToast, type ToastInput } from "$lib/feedback/toasts";
import { getCopy } from "$lib/i18n";
import {
  createRequestId,
  getExplicitErrorMessage,
  toErrorMessage,
} from "$lib/shared/runtime";
import { getWindowsParentPath } from "$lib/shared/windows-path";
import { probeFileSystemPath } from "$lib/tauri/system";
import {
  completeTask,
  failTask,
  longTaskState,
  startTask,
} from "$lib/long-tasks/state";
import {
  exportDistro,
  setDefaultDistro,
  shutdownWsl,
  terminateDistro,
  unregisterDistro,
  type DistroInfo,
  type ExportFormat,
} from "$lib/tauri/wsl";

import { loadLiveDistro, type LiveDistro } from "./live-distro";

let hasSkippedInitialWorkspaceRouteRefresh = false;

interface SubmitExportTaskInput {
  distro: string;
  file: string;
  format: ExportFormat;
  logoSrc: string;
}

export interface DistroWorkspaceService {
  queryCache: Readable<QueryCacheState>;
  vhdSizeCache: Readable<VhdSizeCacheState>;
  getQueryState: () => QueryCacheState;
  getVhdSizeState: () => VhdSizeCacheState;
  enterWorkspace: () => Promise<QueryRefreshResult<DistroInfo[]>>;
  refreshWorkspace: () => Promise<QueryRefreshResult<DistroInfo[]>>;
  refreshWorkspaceAfterAction: () => Promise<QueryRefreshResult<DistroInfo[]>>;
  loadLiveDistro: () => Promise<LiveDistro>;
  getVhdSizeEntry: (
    state: VhdSizeCacheState,
    distro: DistroInfo,
  ) => VhdSizeEntry;
  probeVhdSize: (distro: DistroInfo) => void;
  getInstallLocation: (distro: DistroInfo) => string | null;
  setDefaultDistro: (distro: string) => Promise<void>;
  shutdownWsl: (force: boolean) => Promise<void>;
  terminateDistro: (distro: string) => Promise<void>;
  unregisterDistro: (distro: string) => Promise<void>;
  chooseDirectory: (
    title: string,
    defaultPath?: string,
  ) => Promise<string | null>;
  submitExportTask: (input: SubmitExportTaskInput) => Promise<boolean>;
  confirm: (options: ConfirmDialogOptions) => Promise<string>;
  toast: (input: ToastInput) => void;
  toErrorMessage: (error: unknown) => string;
}

export function createDistroWorkspaceService(): DistroWorkspaceService {
  return {
    queryCache,
    vhdSizeCache,
    getQueryState: () => get(queryCache),
    getVhdSizeState: () => get(vhdSizeCache),
    enterWorkspace: () => {
      if (!hasSkippedInitialWorkspaceRouteRefresh) {
        hasSkippedInitialWorkspaceRouteRefresh = true;
        return Promise.resolve(getCachedWorkspaceResult());
      }

      return refreshDistroWorkspace("page-enter");
    },
    refreshWorkspace: () => refreshDistroWorkspace("manual"),
    refreshWorkspaceAfterAction: refreshDistrosAfterAction,
    loadLiveDistro,
    getVhdSizeEntry,
    probeVhdSize,
    getInstallLocation: buildDistroVhdPath,
    setDefaultDistro,
    shutdownWsl,
    terminateDistro,
    unregisterDistro,
    chooseDirectory,
    submitExportTask,
    confirm: openConfirmDialog,
    toast: pushToast,
    toErrorMessage,
  };
}

async function chooseDirectory(
  title: string,
  defaultPath?: string,
): Promise<string | null> {
  const selected = await open({
    title,
    directory: true,
    multiple: false,
    defaultPath,
  });

  return typeof selected === "string" ? selected : null;
}

async function submitExportTask(input: SubmitExportTaskInput): Promise<boolean> {
  const copy = getCopy();

  if (get(longTaskState).hasActiveLongTask) {
    pushToast({
      tone: "error",
      title: copy.longTasks.exportTasks.activeTitle,
      message: copy.longTasks.exportTasks.activeMessage,
    });
    return false;
  }

  if (!(await guardExportTarget(input))) {
    return false;
  }

  const requestId = createRequestId();
  const noun = copy.common.export;
  let taskStarted = false;
  try {
    await startTask({
      requestId,
      distro: input.distro,
      operation: "export",
      location: input.file,
      logoSrc: input.logoSrc,
    });
    taskStarted = true;
    await exportDistro({
      requestId,
      distro: input.distro,
      file: input.file,
      format: input.format,
    });
    await completeTask(requestId);
    pushToast({
      tone: "success",
      title: copy.longTasks.exportTasks.completedTitle,
      message: copy.longTasks.exportTasks.completedMessage(input.distro),
    });
    return true;
  } catch (error) {
    const message = toErrorMessage(error);
    if (taskStarted) {
      await failTask(requestId, message).catch(() => undefined);
    }
    pushToast({
      tone: "error",
      title: copy.longTasks.exportTasks.failedTitle(noun),
      message,
    });
    return false;
  }
}

async function guardExportTarget(input: SubmitExportTaskInput): Promise<boolean> {
  if (input.format === "Vhd" && !input.file.toLowerCase().endsWith(".vhdx")) {
    const copy = getCopy();
    pushToast({
      tone: "error",
      title: copy.longTasks.exportTasks.targetFileCheckFailedTitle,
      message: copy.longTasks.exportTasks.vhdxSuffixRequiredMessage,
    });
    return false;
  }

  if (!(await guardExportTargetDirectory(input.file))) {
    return false;
  }

  return guardExportTargetFile(input.file);
}

async function guardExportTargetDirectory(file: string): Promise<boolean> {
  const copy = getCopy();
  const directory = getWindowsParentPath(file);

  if (directory === null) {
    pushToast({
      tone: "error",
      title: copy.longTasks.exportTasks.targetDirectoryCheckFailedTitle,
      message: copy.common.errors.operationFailed,
    });
    return false;
  }

  try {
    const probe = await probeFileSystemPath(directory, null);
    if (probe.exists && probe.isDir) {
      return true;
    }

    pushToast({
      tone: "error",
      title: copy.longTasks.exportTasks.targetDirectoryInvalidTitle,
      message: copy.longTasks.exportTasks.targetDirectoryInvalidMessage(directory),
    });
    return false;
  } catch (error) {
    pushToast({
      tone: "error",
      title: copy.longTasks.exportTasks.targetDirectoryCheckFailedTitle,
      message:
        getExplicitErrorMessage(error) ?? copy.common.errors.operationFailed,
    });
    return false;
  }
}

async function guardExportTargetFile(file: string): Promise<boolean> {
  const copy = getCopy();

  try {
    const probe = await probeFileSystemPath(file, null);
    if (!probe.exists) {
      return true;
    }

    pushToast({
      tone: "error",
      title: copy.longTasks.exportTasks.targetFileExistsTitle,
      message: copy.longTasks.exportTasks.targetFileExistsMessage(file),
    });
    return false;
  } catch (error) {
    pushToast({
      tone: "error",
      title: copy.longTasks.exportTasks.targetFileCheckFailedTitle,
      message:
        getExplicitErrorMessage(error) ?? copy.common.errors.operationFailed,
    });
    return false;
  }
}

function getCachedWorkspaceResult(): QueryRefreshResult<DistroInfo[]> {
  const distros = get(queryCache).distros.data;

  if (distros === null) {
    return {
      kind: "failed",
      data: null,
    };
  }

  return {
    kind: "fresh",
    data: distros,
  };
}
