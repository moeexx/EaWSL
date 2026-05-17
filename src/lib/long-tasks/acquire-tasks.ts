import { get } from "svelte/store";

import { checkDistroTargetDirectoryAvailable } from "$lib/shared/distro-targets";
import {
  GENERIC_DISTRO_LOGO_SRC,
  getDistroLogoSrc,
} from "$lib/shared/distro-logos";
import { findDistroByName } from "$lib/shared/distros";
import {
  getQueryErrorMessage,
  queryCache,
  refreshAcquireTaskInBackground as refreshAcquireTaskSideDataInBackground,
  refreshDistrosAfterAction,
} from "$lib/query-cache";
import { pushToast } from "$lib/feedback/toasts";
import { createRequestId, toErrorMessage } from "$lib/shared/runtime";
import { getCopy, type AppCopy } from "$lib/i18n";
import { probeFileSystemPath } from "$lib/tauri/system";
import {
  importDistro,
  importDistroInPlace,
  installDistro,
  type InstallOptions,
} from "$lib/tauri/wsl";

import {
  completeTask,
  failTask,
  longTaskState,
  startTask,
  type LongTaskOperation,
} from "./state";

interface InstallAcquireTask {
  operation: "install";
  distro: string;
  displayName: string;
  location: string;
  options: InstallOptions;
}

interface ImportArchiveAcquireTask {
  operation: "importArchive";
  displayName: string;
  location: string;
  file: string;
}

interface ImportVhdAcquireTask {
  operation: "importVhd";
  displayName: string;
  sourceVhdx: string;
  targetDirectory: string | null;
}

export type SubmitAcquireTaskInput =
  | InstallAcquireTask
  | ImportArchiveAcquireTask
  | ImportVhdAcquireTask;

export async function submitAcquireTask(
  input: SubmitAcquireTaskInput,
): Promise<boolean> {
  if (get(longTaskState).hasActiveLongTask) {
    const copy = getCopy();
    pushToast({
      tone: "error",
      title: copy.longTasks.acquireTasks.activeTitle,
      message: copy.longTasks.acquireTasks.activeMessage,
    });
    return false;
  }

  if (!guardAcquireTaskName(input.displayName)) return false;
  if (!(await guardAcquireTaskTargetDirectory(input))) return false;

  const requestId = createRequestId();
  let taskStarted = false;
  try {
    await startTask({
      requestId,
      distro: input.displayName,
      operation: input.operation,
      location: getAcquireTaskLocation(input),
      logoSrc: getAcquireTaskLogoSrc(input),
    });
    taskStarted = true;
    await runAcquireTaskCommand(requestId, input);
    await completeTask(requestId);
    await syncAcquireTaskResult(input.displayName, input.operation);
    refreshAcquireTaskSideData(input.operation);
    return true;
  } catch (error) {
    const message = toErrorMessage(error);
    const copy = getCopy();
    const noun = getAcquireTaskNoun(input.operation, copy);
    if (taskStarted) {
      await failTask(requestId, message).catch(() => undefined);
    }
    pushToast({
      tone: "error",
      title: copy.longTasks.acquireTasks.failedTitle(noun),
      message,
    });
    return false;
  }
}

function guardAcquireTaskName(name: string): boolean {
  const queryState = get(queryCache);
  const distros = queryState.distros.data;

  if (distros === null) {
    const copy = getCopy();
    pushToast({
      tone: "error",
      title: copy.longTasks.acquireTasks.nameCheckFailedTitle,
      message: getQueryErrorMessage("distros", queryState.distros, copy),
    });
    return false;
  }

  if (findDistroByName(distros, name) !== null) {
    const copy = getCopy();
    pushToast({
      tone: "error",
      title: copy.longTasks.acquireTasks.nameExistsTitle,
      message: copy.longTasks.acquireTasks.nameExistsMessage(name),
    });
    return false;
  }

  return true;
}

async function guardAcquireTaskTargetDirectory(
  input: SubmitAcquireTaskInput,
): Promise<boolean> {
  const location = getPreflightTargetDirectory(input);

  if (location === null) {
    return true;
  }

  const availability = await checkDistroTargetDirectoryAvailable({
    location,
    probeFileSystemPath,
  });

  if (availability.kind === "available") {
    return true;
  }

  if (availability.kind === "exists") {
    const copy = getCopy();
    pushToast({
      tone: "error",
      title: getTargetDirectoryExistsTitle(input.operation, copy),
      message: copy.longTasks.acquireTasks.targetExistsMessage(location),
    });
    return false;
  }

  const copy = getCopy();
  pushToast({
    tone: "error",
    title: copy.longTasks.acquireTasks.directoryCheckFailedTitle,
    message: availability.message ?? copy.common.errors.operationFailed,
  });
  return false;
}

async function runAcquireTaskCommand(
  requestId: string,
  input: SubmitAcquireTaskInput,
): Promise<void> {
  if (input.operation === "install") {
    return installDistro({
      requestId,
      distro: input.distro,
      options: input.options,
    });
  }

  if (input.operation === "importArchive") {
    return importDistro({
      requestId,
      distro: input.displayName,
      location: input.location,
      file: input.file,
    });
  }

  return importDistroInPlace({
    requestId,
    distro: input.displayName,
    sourceVhdx: input.sourceVhdx,
    targetDirectory: input.targetDirectory,
  });
}

async function syncAcquireTaskResult(
  displayName: string,
  operation: LongTaskOperation,
): Promise<void> {
  const result = await refreshDistrosAfterAction();
  const copy = getCopy();
  const noun = getAcquireTaskNoun(operation, copy);

  if (result.kind === "failed") {
    pushToast({
      tone: "warning",
      title: copy.longTasks.acquireTasks.syncFailedTitle(noun),
      message: copy.longTasks.acquireTasks.syncFailedMessage(noun),
    });
    return;
  }

  if (result.kind === "recovering") {
    pushToast({
      tone: "warning",
      title: copy.longTasks.acquireTasks.syncDelayedTitle(noun),
      message: copy.longTasks.acquireTasks.syncDelayedMessage(
        displayName,
        noun,
      ),
    });
    return;
  }

  if (findDistroByName(result.data, displayName) === null) {
    pushToast({
      tone: "warning",
      title: copy.longTasks.acquireTasks.resultUnknownTitle(noun),
      message: copy.longTasks.acquireTasks.resultUnknownMessage(
        displayName,
        noun,
      ),
    });
    return;
  }

  pushToast({
    tone: "success",
    title: copy.longTasks.acquireTasks.completedTitle(noun),
    message: copy.longTasks.acquireTasks.completedMessage(displayName, noun),
  });
}

function refreshAcquireTaskSideData(operation: LongTaskOperation): void {
  refreshAcquireTaskSideDataInBackground(
    operation === "install" ? "install" : "import",
  );
}

function getAcquireTaskLocation(input: SubmitAcquireTaskInput): string | null {
  if (input.operation === "install" || input.operation === "importArchive") {
    return input.location;
  }

  return input.targetDirectory ?? input.sourceVhdx;
}

function getAcquireTaskLogoSrc(input: SubmitAcquireTaskInput): string {
  return input.operation === "install"
    ? getDistroLogoSrc(input.distro)
    : GENERIC_DISTRO_LOGO_SRC;
}

function getPreflightTargetDirectory(
  input: SubmitAcquireTaskInput,
): string | null {
  if (input.operation === "install" || input.operation === "importArchive") {
    return input.location;
  }

  return input.targetDirectory;
}

function getTargetDirectoryExistsTitle(
  operation: LongTaskOperation,
  copy: AppCopy,
): string {
  return operation === "install"
    ? copy.longTasks.acquireTasks.installDirectoryExistsTitle
    : copy.longTasks.acquireTasks.targetDirectoryExistsTitle;
}

function getAcquireTaskNoun(
  operation: LongTaskOperation,
  copy: AppCopy,
): string {
  if (operation === "importArchive")
    return copy.longTasks.operations.importArchive;
  if (operation === "importVhd") return copy.longTasks.operations.importVhd;
  return copy.longTasks.operations.install;
}
