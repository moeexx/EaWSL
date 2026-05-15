import {
  findDistroByName,
  isDistroRunning,
} from "$lib/shared/distros";
import type { QueryRefreshResult } from "$lib/query-cache";
import type { ToastInput } from "$lib/feedback/toasts";
import type { AppCopy } from "$lib/i18n";
import type { DistroInfo, ExportFormat } from "$lib/tauri/wsl";

import type { LiveDistro } from "../service/live-distro";
import {
  createDistroWorkspaceActionFlow,
  getSetDefaultRefreshToast,
  getShutdownRefreshMessages,
  getSyncFailureToast,
  getUnregisterRefreshMessages,
  type DistroWorkspaceActionRuntime,
  type OverlayRefreshMessages,
} from "./distro-workspace-action-support";
import { getWorkspaceActionOverlay } from "./distro-workspace-display";
import {
  createDistroRowScope,
  shutdownAllScope,
  type DistroWorkspaceOverlayScope,
} from "./distro-workspace-types";
import { getActionOverlayMessage } from "./action-overlay";

const SHUTDOWN_ALL_ACTION_BUTTON_KEY = "shutdown-all";

interface PostRefreshToastOptions {
  success: ToastInput;
  failed: ToastInput;
  pendingTitle: string;
  pendingFallback: string;
  unknownTitle: string;
  unknownFallback: string;
  prove: (distros: DistroInfo[]) => boolean;
}

interface ExecuteActionDecision {
  kind: "execute";
  errorTitle: string;
  execute: () => Promise<void>;
  refreshToast?: (result: QueryRefreshResult<DistroInfo[]>) => ToastInput;
  overlayScope?: DistroWorkspaceOverlayScope;
  overlayMessages?: OverlayRefreshMessages;
  preDelayMs?: number;
  postDelayMs?: number;
  onStart?: () => void;
  onFinish?: () => void;
}

type WorkspaceActionDecision =
  | {
      kind: "cancel";
    }
  | {
      kind: "refresh-only";
      refreshToast: (result: QueryRefreshResult<DistroInfo[]>) => ToastInput;
    }
  | ExecuteActionDecision;

export interface DistroWorkspaceActions {
  refresh: () => Promise<void>;
  shutdownAll: () => Promise<void>;
  terminate: (distroName: string) => Promise<void>;
  setDefault: (distroName: string) => Promise<void>;
  unregister: (distroName: string) => Promise<void>;
  chooseExportDirectory: (
    distroName: string,
    defaultPath?: string,
  ) => Promise<string | null>;
  submitExport: (
    distroName: string,
    file: string,
    format: ExportFormat,
  ) => Promise<boolean>;
}

export function createDistroWorkspaceActions(
  runtime: DistroWorkspaceActionRuntime,
): DistroWorkspaceActions {
  const { service } = runtime;
  const flow = createDistroWorkspaceActionFlow(runtime);

  async function loadLiveDistroOrToast(): Promise<LiveDistro | null> {
    const copy = runtime.getCopy();

    try {
      return await service.loadLiveDistro();
    } catch (error) {
      service.toast({
        tone: "error",
        title: copy.distros.actions.liveReadFailedTitle,
        message: service.toErrorMessage(error),
      });
      return null;
    }
  }

  async function runWorkspaceAction(config: {
    buttonKey: string;
    decide: (liveDistro: LiveDistro) => Promise<WorkspaceActionDecision>;
  }): Promise<void> {
    const liveDistro = await loadLiveDistroOrToast();
    if (!liveDistro) return;

    const decision = await config.decide(liveDistro);
    if (decision.kind === "cancel") {
      return;
    }

    runtime.setActiveActionButtonKey(config.buttonKey);

    try {
      if (decision.kind === "refresh-only") {
        const result = await flow.refreshWorkspaceAfterAction();

        if (!runtime.isDisposed()) {
          flow.resolveOverlaysAfterRefresh("action-sync", result);
          service.toast(decision.refreshToast(result));
        }
        return;
      }

      decision.onStart?.();

      if (decision.overlayScope) {
        if (decision.overlayScope.kind === "distro-row") {
          flow.clearRowOverlays(decision.overlayScope.distroName);
        }

        flow.upsertActionOverlay({
          scope: decision.overlayScope,
          phase: "running",
        });
      }

      try {
        await delay(decision.preDelayMs ?? 0);
        await decision.execute();
        await delay(decision.postDelayMs ?? 0);

        const result = await flow.refreshWorkspaceAfterAction();

        if (!runtime.isDisposed()) {
          flow.resolveOverlaysAfterRefresh("action-sync", result);
          service.toast(resolveDecisionToast(decision, result));
        }
      } catch (error) {
        if (!runtime.isDisposed()) {
          if (decision.overlayScope) {
            flow.clearActionOverlay(decision.overlayScope);
          }

          service.toast({
            tone: "error",
            title: decision.errorTitle,
            message: service.toErrorMessage(error),
          });
        }
      } finally {
        if (!runtime.isDisposed()) {
          decision.onFinish?.();
        }
      }
    } finally {
      if (!runtime.isDisposed()) {
        runtime.setActiveActionButtonKey(null);
      }
    }
  }

  async function handleTerminate(distroName: string): Promise<void> {
    await runWorkspaceAction({
      buttonKey: getTerminateActionButtonKey(distroName),
      decide: async (liveDistro) => {
        const copy = runtime.getCopy();

        if (!liveDistro.exists(distroName)) {
          return {
            kind: "refresh-only",
            refreshToast: (result) =>
              resolvePostRefreshToast(
                result,
                buildMissingDistroRefreshToast(
                  distroName,
                  flow.getRecoveringMessage(),
                  copy,
                ),
              ),
          };
        }

        if (liveDistro.isStopped(distroName)) {
          return {
            kind: "refresh-only",
            refreshToast: (result) =>
              resolvePostRefreshToast(
                result,
                buildAlreadyStoppedRefreshToast(
                  distroName,
                  flow.getRecoveringMessage(),
                  copy,
                ),
              ),
          };
        }

        return {
          kind: "execute",
          errorTitle: copy.distros.actions.terminateFailedTitle,
          execute: () => service.terminateDistro(distroName),
          overlayScope: createDistroRowScope(distroName, "terminate"),
          overlayMessages: {
            failed: getSyncFailureToast(
              createDistroRowScope(distroName, "terminate"),
              copy,
            ),
            success: {
              tone: "success",
              title: copy.common.stopped,
              message: copy.distros.actions.terminateSuccessMessage(distroName),
            },
            pendingTitle: copy.distros.actions.terminatePendingTitle,
            pendingFallback: flow.getRecoveringMessage(),
            unknownTitle: copy.distros.actions.terminateUnknownTitle,
            unknownFallback:
              copy.distros.actions.terminateUnknownFallback(distroName),
          },
          preDelayMs: 100,
          postDelayMs: 100,
        };
      },
    });
  }

  async function handleShutdownAll(): Promise<void> {
    await runWorkspaceAction({
      buttonKey: SHUTDOWN_ALL_ACTION_BUTTON_KEY,
      decide: async (liveDistro) => {
        const copy = runtime.getCopy();

        if (!liveDistro.hasAnyRunning()) {
          return {
            kind: "refresh-only",
            refreshToast: (result) =>
              resolvePostRefreshToast(
                result,
                buildNoRunningDistroRefreshToast(
                  flow.getRecoveringMessage(),
                  copy,
                ),
              ),
          };
        }

        const selection = await service.confirm({
          title: copy.distros.actions.shutdownDialogTitle,
          message: copy.distros.actions.shutdownDialogMessage,
          tone: "warning",
          actions: [
            {
              id: "cancel",
              label: copy.distros.actions.cancel,
              variant: "secondary",
            },
            { id: "stop", label: copy.common.stop, variant: "primary" },
            {
              id: "force",
              label: copy.distros.actions.forceStop,
              variant: "danger",
            },
          ],
        });

        if (selection === "cancel") {
          return {
            kind: "cancel",
          };
        }

        const force = selection === "force";

        return {
          kind: "execute",
          errorTitle: force
            ? copy.distros.actions.forceStopFailedTitle
            : copy.distros.actions.stopAllFailedTitle,
          execute: () => service.shutdownWsl(force),
          overlayScope: shutdownAllScope,
          overlayMessages: getShutdownRefreshMessages(
            force,
            flow.getRecoveringMessage(),
            copy,
          ),
          preDelayMs: 100,
          postDelayMs: 100,
          onStart: () => {
            runtime.setShutdownMode(force ? "force" : "normal");
          },
          onFinish: () => {
            runtime.setShutdownMode(null);
          },
        };
      },
    });
  }

  async function handleSetDefault(distroName: string): Promise<void> {
    await runWorkspaceAction({
      buttonKey: getSetDefaultActionButtonKey(distroName),
      decide: async (liveDistro) => {
        const copy = runtime.getCopy();

        if (!liveDistro.exists(distroName)) {
          return {
            kind: "refresh-only",
            refreshToast: (result) =>
              resolvePostRefreshToast(
                result,
                buildMissingDistroRefreshToast(
                  distroName,
                  flow.getRecoveringMessage(),
                  copy,
                ),
              ),
          };
        }

        if (liveDistro.isDefault(distroName)) {
          return {
            kind: "refresh-only",
            refreshToast: (result) =>
              resolvePostRefreshToast(
                result,
                buildAlreadyDefaultRefreshToast(
                  distroName,
                  flow.getRecoveringMessage(),
                  copy,
                ),
              ),
          };
        }

        return {
          kind: "execute",
          errorTitle: copy.distros.actions.setDefaultFailedTitle,
          execute: () => service.setDefaultDistro(distroName),
          preDelayMs: 0,
          postDelayMs: 100,
          refreshToast: (result) =>
            getSetDefaultRefreshToast(
              result,
              distroName,
              flow.getRecoveringMessage(),
              copy,
            ),
        };
      },
    });
  }

  async function handleUnregister(distroName: string): Promise<void> {
    await runWorkspaceAction({
      buttonKey: getUnregisterActionButtonKey(distroName),
      decide: async (liveDistro) => {
        const copy = runtime.getCopy();

        if (!liveDistro.exists(distroName)) {
          return {
            kind: "refresh-only",
            refreshToast: (result) =>
              resolvePostRefreshToast(
                result,
                buildMissingDistroRefreshToast(
                  distroName,
                  flow.getRecoveringMessage(),
                  copy,
                ),
              ),
          };
        }

        const selection = await service.confirm({
          title: copy.distros.actions.unregisterDialogTitle(distroName),
          message: copy.distros.actions.unregisterDialogMessage(distroName),
          tone: "danger",
          actions: [
            {
              id: "cancel",
              label: copy.distros.actions.cancel,
              variant: "secondary",
            },
            {
              id: "confirm",
              label: copy.distros.row.delete,
              variant: "danger",
            },
          ],
        });

        if (selection !== "confirm") {
          return {
            kind: "cancel",
          };
        }

        return {
          kind: "execute",
          errorTitle: copy.distros.actions.unregisterFailedTitle,
          execute: () => service.unregisterDistro(distroName),
          overlayScope: createDistroRowScope(distroName, "unregister"),
          overlayMessages: getUnregisterRefreshMessages(
            distroName,
            flow.getRecoveringMessage(),
            copy,
          ),
          postDelayMs: 100,
        };
      },
    });
  }

  async function chooseExportDirectory(
    distroName: string,
    defaultPath?: string,
  ): Promise<string | null> {
    return service.chooseDirectory(
      runtime.getCopy().distros.row.exportMenu.directoryDialogTitle(distroName),
      defaultPath,
    );
  }

  async function submitExport(
    distroName: string,
    file: string,
    format: ExportFormat,
  ): Promise<boolean> {
    return service.submitExportTask({
      distro: distroName,
      file,
      format,
    });
  }

  function resolveDecisionToast(
    decision: ExecuteActionDecision,
    result: QueryRefreshResult<DistroInfo[]>,
  ): ToastInput {
    if (decision.overlayScope && decision.overlayMessages) {
      return resolveOverlayRefreshToast(
        decision.overlayScope,
        result,
        decision.overlayMessages,
      );
    }

    if (decision.refreshToast) {
      return decision.refreshToast(result);
    }

    return {
      tone: "warning",
      title: runtime.getCopy().distros.actions.syncUnknownTitle,
      message: runtime.getCopy().distros.actions.syncUnknownMessage,
    };
  }

  return {
    refresh: flow.handleManualRefresh,
    shutdownAll: handleShutdownAll,
    terminate: handleTerminate,
    setDefault: handleSetDefault,
    unregister: handleUnregister,
    chooseExportDirectory,
    submitExport,
  };

  function resolveOverlayRefreshToast(
    scope: DistroWorkspaceOverlayScope,
    result: QueryRefreshResult<DistroInfo[]>,
    messages: OverlayRefreshMessages,
  ): ToastInput {
    if (result.kind === "failed") {
      return messages.failed;
    }

    const overlay = getWorkspaceActionOverlay(runtime.getActionOverlays(), scope);
    if (!overlay) {
      return messages.success;
    }

    return {
      tone: "warning",
      title:
        overlay.phase === "pending"
          ? messages.pendingTitle
          : messages.unknownTitle,
      message:
        getActionOverlayMessage(overlay, runtime.getCopy()) ??
        (overlay.phase === "pending"
          ? messages.pendingFallback
          : messages.unknownFallback),
    };
  }
}

function buildMissingDistroRefreshToast(
  distroName: string,
  recoveringMessage: string,
  copy: AppCopy,
): PostRefreshToastOptions {
  return {
    success: {
      tone: "warning",
      title: copy.distros.actions.missingDistroTitle,
      message: copy.distros.actions.missingDistroSuccess(distroName),
    },
    failed: {
      tone: "warning",
      title: copy.distros.actions.missingDistroTitle,
      message: copy.distros.actions.missingDistroFailed(distroName),
    },
    pendingTitle: copy.distros.actions.listStatePendingTitle,
    pendingFallback: recoveringMessage,
    unknownTitle: copy.distros.actions.distroChangedTitle,
    unknownFallback: copy.distros.actions.distroChangedFallback(distroName),
    prove: (distros) => findDistroByName(distros, distroName) === null,
  };
}

function buildAlreadyStoppedRefreshToast(
  distroName: string,
  recoveringMessage: string,
  copy: AppCopy,
): PostRefreshToastOptions {
  return {
    success: {
      tone: "success",
      title: copy.distros.actions.noStopNeededTitle,
      message: copy.distros.actions.alreadyStoppedMessage(distroName),
    },
    failed: {
      tone: "warning",
      title: copy.distros.actions.stopSyncFailedTitle,
      message: copy.distros.actions.alreadyStoppedFailed(distroName),
    },
    pendingTitle: copy.distros.actions.terminatePendingTitle,
    pendingFallback: recoveringMessage,
    unknownTitle: copy.distros.actions.terminateUnknownTitle,
    unknownFallback: copy.distros.actions.alreadyStoppedUnknown(distroName),
    prove: (distros) => {
      const distro = findDistroByName(distros, distroName);
      return distro !== null && !isDistroRunning(distro);
    },
  };
}

function buildNoRunningDistroRefreshToast(
  recoveringMessage: string,
  copy: AppCopy,
): PostRefreshToastOptions {
  return {
    success: {
      tone: "success",
      title: copy.distros.actions.noStopNeededTitle,
      message: copy.distros.actions.noRunningMessage,
    },
    failed: {
      tone: "warning",
      title: copy.distros.actions.stopSyncFailedTitle,
      message: copy.distros.actions.noRunningFailed,
    },
    pendingTitle: copy.distros.actions.terminatePendingTitle,
    pendingFallback: recoveringMessage,
    unknownTitle: copy.distros.actions.terminateUnknownTitle,
    unknownFallback: copy.distros.actions.noRunningUnknown,
    prove: (distros) => !distros.some(isDistroRunning),
  };
}

function buildAlreadyDefaultRefreshToast(
  distroName: string,
  recoveringMessage: string,
  copy: AppCopy,
): PostRefreshToastOptions {
  return {
    success: {
      tone: "success",
      title: copy.distros.actions.noDefaultNeededTitle,
      message: copy.distros.actions.alreadyDefaultMessage(distroName),
    },
    failed: {
      tone: "warning",
      title: copy.distros.actionSupport.setDefaultSyncFailedTitle,
      message: copy.distros.actions.alreadyDefaultFailed(distroName),
    },
    pendingTitle: copy.distros.actions.setDefaultPendingTitle,
    pendingFallback: recoveringMessage,
    unknownTitle: copy.distros.actions.setDefaultUnknownTitle,
    unknownFallback: copy.distros.actions.setDefaultChangedFallback(distroName),
    prove: (distros) =>
      findDistroByName(distros, distroName)?.is_default === true,
  };
}

function resolvePostRefreshToast(
  result: QueryRefreshResult<DistroInfo[]>,
  options: PostRefreshToastOptions,
): ToastInput {
  if (result.kind === "failed") {
    return options.failed;
  }

  if (result.kind === "fresh" && options.prove(result.data)) {
    return options.success;
  }

  return {
    tone: "warning",
    title:
      result.kind === "recovering"
        ? options.pendingTitle
        : options.unknownTitle,
    message:
      result.kind === "recovering"
        ? options.pendingFallback
        : options.unknownFallback,
  };
}

function getTerminateActionButtonKey(distroName: string): string {
  return `terminate:${distroName}`;
}

function getSetDefaultActionButtonKey(distroName: string): string {
  return `set-default:${distroName}`;
}

function getUnregisterActionButtonKey(distroName: string): string {
  return `unregister:${distroName}`;
}

function delay(ms: number): Promise<void> {
  if (ms <= 0) {
    return Promise.resolve();
  }

  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
