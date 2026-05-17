import type { SystemOverview, SystemOverviewScope } from "$lib/tauri/system";
import type { DistroInfo, OnlineDistro, WslVersion } from "$lib/tauri/wsl";
import type { RefreshReason } from "$lib/shared/refresh-reasons";
import type { RecoverableCommandCode } from "$lib/tauri/errors";

export type QueryKey =
  | "distros"
  | "wslVersion"
  | "systemOverview"
  | "onlineDistros";

export type QueryRequest =
  | Exclude<QueryKey, "systemOverview">
  | "systemOverview"
  | {
      key: "systemOverview";
      scope: SystemOverviewScope;
    };

export type QueryRefreshReason = RefreshReason;

export type QueryIntent = QueryRefreshReason;

export type QueryActivity = "idle" | "loading" | "refreshing";

export interface QueryEntry<T> {
  data: T | null;
  hasError: boolean;
  errorMessage: string | null;
  activity: QueryActivity;
  lastSuccessAt: Date | null;
  expiresAt: Date | null;
  lastFullSuccessAt: Date | null;
  fullExpiresAt: Date | null;
  isRecovering: boolean;
  recoveringState: QueryRecoveringState | null;
}

export interface QueryRecoveringState {
  code: RecoverableCommandCode;
  hasSuccessfulData: boolean;
}

export type QueryRefreshResult<T> =
  | {
      kind: "fresh";
      data: T;
    }
  | {
      kind: "recovering";
      data: T | null;
    }
  | {
      kind: "failed";
      data: null;
    };

export type QueryDataByKey = {
  distros: DistroInfo[];
  wslVersion: WslVersion;
  systemOverview: SystemOverview;
  onlineDistros: OnlineDistro[];
};

export type QueryRefreshResults = Partial<{
  [K in QueryKey]: QueryRefreshResult<QueryDataByKey[K]>;
}>;

export interface QueryRefreshPlan {
  foreground: readonly QueryRequest[];
  background: readonly QueryRequest[];
  reason: QueryRefreshReason;
  foregroundMinDurationMs?: number;
}

export interface QueryCacheState {
  distros: QueryEntry<DistroInfo[]>;
  wslVersion: QueryEntry<WslVersion>;
  systemOverview: QueryEntry<SystemOverview>;
  onlineDistros: QueryEntry<OnlineDistro[]>;
}
