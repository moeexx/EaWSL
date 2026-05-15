import { findDistroByName } from "$lib/shared/distros";
import type { QueryIntent, QueryRefreshResult } from "$lib/query-cache";
import type { ToastInput } from "$lib/feedback/toasts";
import type { AppCopy } from "$lib/i18n";
import type { DistroInfo } from "$lib/tauri/wsl";

import type { DistroWorkspaceService } from "../service/distro-workspace-service";
import { getDistroRecoveringMessage } from "./distro-workspace-display";
import type {
  DistroWorkspaceActionButtonKey,
  DistroWorkspaceOverlayScope,
  DistroWorkspaceOverlayState,
  ShutdownMode,
} from "./distro-workspace-types";
import {
  isSameActionOverlayScope,
  resolveActionOverlayWithDistrosRefresh,
} from "./action-overlay";

export interface DistroWorkspaceActionRuntime {
  service: DistroWorkspaceService;
  getCopy: () => AppCopy;
  isDisposed: () => boolean;
  getActionOverlays: () => DistroWorkspaceOverlayState[];
  setActionOverlays: (overlays: DistroWorkspaceOverlayState[]) => void;
  setShutdownMode: (mode: ShutdownMode) => void;
  setActiveActionButtonKey: (key: DistroWorkspaceActionButtonKey) => void;
}

export interface OverlayRefreshMessages {
  failed: ToastInput;
  success: ToastInput;
  pendingTitle: string;
  pendingFallback: string;
  unknownTitle: string;
  unknownFallback: string;
}

export interface DistroWorkspaceActionFlow {
  refreshWorkspaceAfterAction: () => Promise<QueryRefreshResult<DistroInfo[]>>;
  upsertActionOverlay: (overlay: DistroWorkspaceOverlayState) => void;
  clearActionOverlay: (scope: DistroWorkspaceOverlayScope) => void;
  clearRowOverlays: (distroName: string) => void;
  resolveOverlaysAfterRefresh: (
    intent: QueryIntent,
    result: QueryRefreshResult<DistroInfo[]>,
  ) => void;
  handleManualRefresh: () => Promise<void>;
  getRecoveringMessage: () => string;
}

export function createDistroWorkspaceActionFlow(
  runtime: DistroWorkspaceActionRuntime,
): DistroWorkspaceActionFlow {
  const { service } = runtime;

  async function refreshWorkspaceAfterAction(): Promise<
    QueryRefreshResult<DistroInfo[]>
  > {
    return service.refreshWorkspaceAfterAction();
  }

  function getRecoveringMessage(): string {
    return getDistroRecoveringMessage(service.getQueryState(), runtime.getCopy());
  }

  function setOverlays(
    updater: (
      overlays: DistroWorkspaceOverlayState[],
    ) => DistroWorkspaceOverlayState[],
  ): void {
    runtime.setActionOverlays(updater(runtime.getActionOverlays()));
  }

  function clearActionOverlay(scope: DistroWorkspaceOverlayScope): void {
    setOverlays((overlays) =>
      overlays.filter((overlay) => !isSameActionOverlayScope(overlay.scope, scope)),
    );
  }

  function clearRowOverlays(distroName: string): void {
    setOverlays((overlays) =>
      overlays.filter(
        (overlay) =>
          overlay.scope.kind !== "distro-row" ||
          overlay.scope.distroName !== distroName,
      ),
    );
  }

  function upsertActionOverlay(nextOverlay: DistroWorkspaceOverlayState): void {
    if (nextOverlay.phase === "clear") {
      clearActionOverlay(nextOverlay.scope);
      return;
    }

    setOverlays((overlays) => [
      ...overlays.filter(
        (overlay) => !isSameActionOverlayScope(overlay.scope, nextOverlay.scope),
      ),
      nextOverlay,
    ]);
  }

  function resolveAllOverlaysAfterRefresh(
    intent: QueryIntent,
    result: QueryRefreshResult<DistroInfo[]>,
  ): void {
    setOverlays((overlays) =>
      overlays.flatMap((overlay) => {
        const nextOverlay = resolveActionOverlayWithDistrosRefresh(
          overlay,
          intent,
          result,
        );
        return nextOverlay.phase === "clear"
          ? []
          : [nextOverlay as DistroWorkspaceOverlayState];
      }),
    );
  }

  async function handleManualRefresh(): Promise<void> {
    const result = await service.refreshWorkspace();
    if (!runtime.isDisposed()) {
      resolveAllOverlaysAfterRefresh("manual", result);
    }
  }

  return {
    refreshWorkspaceAfterAction,
    upsertActionOverlay,
    clearActionOverlay,
    clearRowOverlays,
    resolveOverlaysAfterRefresh: resolveAllOverlaysAfterRefresh,
    handleManualRefresh,
    getRecoveringMessage,
  };
}

export function getSyncFailureToast(
  scope: DistroWorkspaceOverlayScope,
  copy: AppCopy,
): ToastInput {
  return scope.kind === "distros-list"
    ? {
        tone: "warning",
        title: copy.distros.actionSupport.stopSyncFailedTitle,
        message: copy.distros.actionSupport.stopSyncFailedMessage,
      }
    : {
        tone: "warning",
        title: copy.distros.actionSupport.actionSyncFailedTitle,
        message: copy.distros.actionSupport.actionSyncFailedMessage,
      };
}

export function getShutdownRefreshMessages(
  force: boolean,
  recoveringMessage: string,
  copy: AppCopy,
): OverlayRefreshMessages {
  return {
    failed: {
      tone: "warning",
      title: force
        ? copy.distros.actionSupport.forceStopSyncFailedTitle
        : copy.distros.actionSupport.stopSyncFailedTitle,
      message: copy.distros.actionSupport.shutdownSyncFailedMessage,
    },
    success: {
      tone: "success",
      title: force
        ? copy.distros.actionSupport.forceStoppedTitle
        : copy.distros.actionSupport.stoppedAllTitle,
      message: force
        ? copy.distros.actionSupport.forceStoppedMessage
        : copy.distros.actionSupport.stoppedAllMessage,
    },
    pendingTitle: force
      ? copy.distros.actionSupport.forceStopPendingTitle
      : copy.distros.actionSupport.stopPendingTitle,
    pendingFallback: recoveringMessage,
    unknownTitle: force
      ? copy.distros.actionSupport.forceStopUnknownTitle
      : copy.distros.actionSupport.stopUnknownTitle,
    unknownFallback: copy.distros.actionSupport.stopUnknownFallback,
  };
}

export function getUnregisterRefreshMessages(
  distroName: string,
  recoveringMessage: string,
  copy: AppCopy,
): OverlayRefreshMessages {
  return {
    failed: {
      tone: "warning",
      title: copy.distros.actionSupport.unregisterSyncFailedTitle,
      message: copy.distros.actionSupport.unregisterSyncFailedMessage,
    },
    success: {
      tone: "success",
      title: copy.distros.actionSupport.unregisterSuccessTitle,
      message: copy.distros.actionSupport.unregisterSuccessMessage(distroName),
    },
    pendingTitle: copy.distros.actionSupport.unregisterPendingTitle,
    pendingFallback: recoveringMessage,
    unknownTitle: copy.distros.actionSupport.unregisterUnknownTitle,
    unknownFallback: copy.distros.actionSupport.unregisterUnknownFallback,
  };
}

export function getSetDefaultRefreshToast(
  result: QueryRefreshResult<DistroInfo[]>,
  distroName: string,
  recoveringMessage: string,
  copy: AppCopy,
): ToastInput {
  if (result.kind === "failed") {
    return {
      tone: "warning",
      title: copy.distros.actionSupport.setDefaultSyncFailedTitle,
      message: copy.distros.actionSupport.setDefaultSyncFailedMessage,
    };
  }

  if (
    result.kind === "fresh" &&
    findDistroByName(result.data, distroName)?.is_default === true
  ) {
    return {
      tone: "success",
      title: copy.distros.actionSupport.setDefaultSuccessTitle,
      message: copy.distros.actionSupport.setDefaultSuccessMessage(distroName),
    };
  }

  return {
    tone: "warning",
    title:
      result.kind === "recovering"
        ? copy.distros.actions.setDefaultPendingTitle
        : copy.distros.actionSupport.setDefaultUnknownTitle,
    message:
      result.kind === "recovering"
        ? recoveringMessage
        : copy.distros.actionSupport.setDefaultUnknownFallback(distroName),
  };
}
