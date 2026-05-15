import type { QueryIntent, QueryRefreshResult } from "$lib/query-cache";
import type { AppCopy } from "$lib/i18n";
import type { DistroInfo } from "$lib/tauri/wsl";

import { findDistroByName, isDistroRunning } from "$lib/shared/distros";

export type ActionOverlayPhase =
  | "clear"
  | "running"
  | "verifying"
  | "pending"
  | "unknown";

export type ActionOverlayScope =
  | {
      kind: "distros-list";
      operation: "shutdown-all";
    }
  | {
      kind: "distro-row";
      distroName: string;
      operation: "terminate" | "unregister";
    };

export interface ActionOverlayState {
  scope: ActionOverlayScope;
  phase: ActionOverlayPhase;
}

type RelevantRefreshIntent = "action-sync" | "manual";

function isRelevantActionRefreshIntent(
  intent: QueryIntent,
): intent is RelevantRefreshIntent {
  return intent === "action-sync" || intent === "manual";
}

export function isSameActionOverlayScope(
  left: ActionOverlayScope,
  right: ActionOverlayScope,
): boolean {
  if (left.kind !== right.kind || left.operation !== right.operation) {
    return false;
  }

  if (left.kind === "distros-list" && right.kind === "distros-list") {
    return true;
  }

  if (left.kind === "distro-row" && right.kind === "distro-row") {
    return left.distroName === right.distroName;
  }

  return false;
}

export function isBlockingActionOverlayPhase(phase: ActionOverlayPhase): boolean {
  return phase === "running" || phase === "verifying";
}

export function resolveActionOverlayWithDistrosRefresh(
  current: ActionOverlayState,
  intent: QueryIntent,
  result: QueryRefreshResult<DistroInfo[]>,
): ActionOverlayState {
  if (!isRelevantActionRefreshIntent(intent)) {
    return current;
  }

  if (result.kind === "failed") {
    return current;
  }

  if (result.kind === "recovering") {
    if (current.phase === "running" || current.phase === "verifying") {
      return {
        scope: current.scope,
        phase: "pending",
      };
    }

    return current;
  }

  if (doesFreshSnapshotProveScope(current.scope, result.data)) {
    return {
      scope: current.scope,
      phase: "clear",
    };
  }

  if (
    current.phase === "running" ||
    current.phase === "verifying" ||
    current.phase === "pending"
  ) {
    return {
      scope: current.scope,
      phase: "unknown",
    };
  }

  return current;
}

function doesFreshSnapshotProveScope(
  scope: ActionOverlayScope,
  distros: DistroInfo[],
): boolean {
  switch (scope.kind) {
    case "distros-list":
      return !distros.some(isDistroRunning);
    case "distro-row":
      return doesFreshSnapshotProveRowScope(scope, distros);
  }
}

function doesFreshSnapshotProveRowScope(
  scope: Extract<ActionOverlayScope, { kind: "distro-row" }>,
  distros: DistroInfo[],
): boolean {
  const targetDistro = findDistroByName(distros, scope.distroName);

  switch (scope.operation) {
    case "terminate":
      return targetDistro !== null && !isDistroRunning(targetDistro);
    case "unregister":
      return targetDistro === null;
  }
}

export function getActionOverlayMessage(
  overlay: ActionOverlayState,
  copy: AppCopy,
): string | null {
  switch (overlay.phase) {
    case "pending":
      return getPendingMessage(overlay.scope, copy);
    case "unknown":
      return getUnknownMessage(overlay.scope, copy);
    default:
      return null;
  }
}

function getPendingMessage(scope: ActionOverlayScope, copy: AppCopy): string {
  switch (scope.kind) {
    case "distros-list":
      return copy.distros.overlays.pendingShutdown;
    case "distro-row":
      switch (scope.operation) {
        case "terminate":
          return copy.distros.overlays.pendingTerminate(scope.distroName);
        case "unregister":
          return copy.distros.overlays.pendingUnregister(scope.distroName);
      }
  }
}

function getUnknownMessage(scope: ActionOverlayScope, copy: AppCopy): string {
  switch (scope.kind) {
    case "distros-list":
      return copy.distros.overlays.unknownShutdown;
    case "distro-row":
      switch (scope.operation) {
        case "terminate":
          return copy.distros.overlays.unknownTerminate(scope.distroName);
        case "unregister":
          return copy.distros.overlays.unknownUnregister(scope.distroName);
      }
  }
}
