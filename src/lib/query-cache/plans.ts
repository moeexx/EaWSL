import type { DistroInfo } from "$lib/tauri/wsl";
import { runLoggedRefreshFlow } from "$lib/shared/frontend-logs";

import { refreshQueries } from "./orchestrator";
import type {
  QueryRefreshReason,
  QueryRefreshResult,
  QueryRefreshResults,
} from "./types";

export type WorkspaceRefreshReason = Extract<
  QueryRefreshReason,
  "page-enter" | "manual"
>;

export type AcquireTaskBackgroundRefresh = "install" | "import";

export function refreshAcquireWorkspace(
  reason: WorkspaceRefreshReason,
): Promise<QueryRefreshResults> {
  return runLoggedRefreshFlow("Install and import workspace", reason, () =>
    refreshQueries({
      foreground: ["onlineDistros"],
      background: [{ key: "systemOverview", scope: "storage" }],
      reason,
      foregroundMinDurationMs: reason === "manual" ? 500 : 0,
    }),
  );
}

export async function refreshDistroWorkspace(
  reason: WorkspaceRefreshReason,
): Promise<QueryRefreshResult<DistroInfo[]>> {
  const results = await runLoggedRefreshFlow("Core workspace", reason, () =>
    refreshQueries({
      foreground: ["distros"],
      background: [{ key: "systemOverview", scope: "storage" }],
      reason,
      foregroundMinDurationMs: reason === "manual" ? 500 : 0,
    }),
  );

  return results.distros ?? { kind: "failed", data: null };
}

export async function refreshDistrosAfterAction(): Promise<
  QueryRefreshResult<DistroInfo[]>
> {
  return syncDistrosAfterMutation();
}

export async function syncDistrosAfterMutation(): Promise<
  QueryRefreshResult<DistroInfo[]>
> {
  const results = await refreshQueries({
    foreground: ["distros"],
    background: [],
    reason: "action-sync",
    foregroundMinDurationMs: 0,
  });

  return results.distros ?? { kind: "failed", data: null };
}

export function refreshAcquireTaskInBackground(
  target: AcquireTaskBackgroundRefresh,
): void {
  if (target === "install") {
    void refreshQueries({
      foreground: [],
      background: ["onlineDistros"],
      reason: "background",
    });
    return;
  }

  void refreshQueries({
    foreground: [],
    background: [{ key: "systemOverview", scope: "storage" }],
    reason: "background",
  });
}
