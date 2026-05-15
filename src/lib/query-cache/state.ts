import { get, writable } from "svelte/store";

import type {
  SystemOverview,
  SystemOverviewScope,
} from "$lib/tauri/system";
import type {
  DistroInfo,
  OnlineDistro,
  WslVersion,
} from "$lib/tauri/wsl";
import type { RecoverableCommandError } from "$lib/tauri/errors";
import { clearVhdSizeCache } from "$lib/probes/distro-vhd-size";
import { getExplicitErrorMessage } from "$lib/shared/runtime";
import type {
  QueryCacheState,
  QueryDataByKey,
  QueryEntry,
  QueryKey,
  QueryRefreshReason,
  QueryRefreshResult,
} from "./types";

const QUERY_CACHE_TTL_MS = 60 * 60 * 1000;

const forceReasons = new Set<QueryRefreshReason>([
  "manual",
  "action-sync",
  "background",
]);

export type ResolvedQueryRequest = {
  key: QueryKey;
  scope: SystemOverviewScope | null;
};

function createQueryEntry<T>(): QueryEntry<T> {
  return {
    data: null,
    hasError: false,
    errorMessage: null,
    activity: "loading",
    lastSuccessAt: null,
    expiresAt: null,
    lastFullSuccessAt: null,
    fullExpiresAt: null,
    isRecovering: false,
    recoveringState: null,
  };
}

const initialState: QueryCacheState = {
  distros: createQueryEntry<DistroInfo[]>(),
  wslVersion: createQueryEntry<WslVersion>(),
  systemOverview: createQueryEntry<SystemOverview>(),
  onlineDistros: createQueryEntry<OnlineDistro[]>(),
};

const store = writable<QueryCacheState>(initialState);

export const queryCache = {
  subscribe: store.subscribe,
};

export function readQueryEntry<K extends QueryKey>(
  key: K,
): QueryEntry<QueryDataByKey[K]> {
  return getCacheEntry(get(store), key);
}

export function shouldRequest<T>(
  entry: QueryEntry<T>,
  reason: QueryRefreshReason,
  request: ResolvedQueryRequest,
): boolean {
  return forceReasons.has(reason) || !hasFreshCache(entry, request);
}

export function getFreshSinceResult<K extends QueryKey>(
  entry: QueryEntry<QueryDataByKey[K]>,
  timestamp: number | null | undefined,
  request: ResolvedQueryRequest,
): QueryRefreshResult<QueryDataByKey[K]> | null {
  const lastSuccessAt =
    request.key === "systemOverview" && request.scope === "full"
      ? entry.lastFullSuccessAt
      : entry.lastSuccessAt;

  if (
    timestamp === null ||
    timestamp === undefined ||
    entry.data === null ||
    lastSuccessAt === null ||
    lastSuccessAt.getTime() < timestamp
  ) {
    return null;
  }

  return freshResult(entry.data);
}

function freshResult<T>(data: T): QueryRefreshResult<T> {
  return {
    kind: "fresh",
    data,
  };
}

function failedResult<T>(): QueryRefreshResult<T> {
  return {
    kind: "failed",
    data: null,
  };
}

export function markQueryStarted<K extends QueryKey>(
  key: K,
  reason: QueryRefreshReason,
): void {
  updateCacheEntry(key, (entry) => {
    const base = clearRecovering(entry);

    if (reason === "startup" || reason === "page-enter") {
      return {
        ...base,
        hasError: false,
        errorMessage: null,
        activity: entry.data === null ? "loading" : entry.activity,
      };
    }

    if (reason === "manual") {
      return {
        ...base,
        hasError: false,
        errorMessage: null,
        activity: "refreshing",
      };
    }

    return base;
  });
}

export function writeQuerySuccess<K extends QueryKey>(
  key: K,
  data: QueryDataByKey[K],
  request: ResolvedQueryRequest,
): QueryRefreshResult<QueryDataByKey[K]> {
  const lastSuccessAt = new Date();
  const expiresAt = new Date(lastSuccessAt.getTime() + QUERY_CACHE_TTL_MS);

  let resolvedData: QueryDataByKey[K] = data;

  updateCacheEntry(key, (entry) => {
    resolvedData = resolveNextQueryData(key, entry.data, data, request);

    return {
      ...entry,
      data: resolvedData,
      hasError: false,
      errorMessage: null,
      activity: "idle",
      lastSuccessAt,
      expiresAt,
      lastFullSuccessAt:
        key === "systemOverview" && request.scope !== "full"
          ? entry.lastFullSuccessAt
          : lastSuccessAt,
      fullExpiresAt:
        key === "systemOverview" && request.scope !== "full"
          ? entry.fullExpiresAt
          : expiresAt,
      isRecovering: false,
      recoveringState: null,
    };
  });

  if (key === "distros") {
    clearVhdSizeCache();
  }

  return freshResult(resolvedData);
}

export function writeQueryFailure<K extends QueryKey>(
  key: K,
  error: unknown,
  reason: QueryRefreshReason,
): QueryRefreshResult<QueryDataByKey[K]> {
  updateCacheEntry(key, (entry) => {
    const nextEntry = idleWithoutRecovering(entry);

    if (reason === "background") {
      return nextEntry;
    }

    return {
      ...nextEntry,
      hasError: true,
      errorMessage: getExplicitErrorMessage(error),
    };
  });

  return failedResult();
}

export function writeQueryRecovering<K extends QueryKey>(
  key: K,
  error: RecoverableCommandError,
  reason: QueryRefreshReason,
): {
  result: QueryRefreshResult<QueryDataByKey[K]>;
  fallbackData: QueryDataByKey[K] | null;
} {
  let fallbackData: QueryDataByKey[K] | null = null;

  updateCacheEntry(key, (entry) => {
    fallbackData = entry.data;

    if (reason === "background") {
      return {
        ...entry,
        activity: "idle",
      };
    }

    return {
      ...entry,
      activity: "idle",
      hasError: false,
      errorMessage: null,
      isRecovering: true,
      recoveringState: {
        code: error.code,
        hasSuccessfulData: entry.data !== null && entry.lastSuccessAt !== null,
      },
    };
  });

  return {
    result: {
      kind: "recovering",
      data: fallbackData,
    },
    fallbackData,
  };
}

export function resolveEffectiveQueryRequest<K extends QueryKey>(
  request: ResolvedQueryRequest,
  entry: QueryEntry<QueryDataByKey[K]>,
): ResolvedQueryRequest {
  if (
    request.key === "systemOverview" &&
    request.scope === "storage" &&
    entry.data === null
  ) {
    return {
      key: request.key,
      scope: "full",
    };
  }

  return request;
}

function getCacheEntry<K extends QueryKey>(
  state: QueryCacheState,
  key: K,
): QueryEntry<QueryDataByKey[K]> {
  return state[key] as QueryEntry<QueryDataByKey[K]>;
}

function updateCacheEntry<K extends QueryKey>(
  key: K,
  updater: (entry: QueryEntry<QueryDataByKey[K]>) => QueryEntry<QueryDataByKey[K]>,
): void {
  store.update((state) => ({
    ...state,
    [key]: updater(getCacheEntry(state, key)),
  }) as QueryCacheState);
}

function hasFreshCache<T>(
  entry: QueryEntry<T>,
  request: ResolvedQueryRequest,
  now = new Date(),
): entry is QueryEntry<T> & { data: T } {
  const expiresAt =
    request.key === "systemOverview" && request.scope === "full"
      ? entry.fullExpiresAt
      : entry.expiresAt;

  return (
    entry.data !== null &&
    expiresAt !== null &&
    expiresAt.getTime() > now.getTime()
  );
}

function clearRecovering<T>(entry: QueryEntry<T>): QueryEntry<T> {
  return {
    ...entry,
    isRecovering: false,
    recoveringState: null,
  };
}

function idleWithoutRecovering<T>(entry: QueryEntry<T>): QueryEntry<T> {
  return {
    ...clearRecovering(entry),
    activity: "idle",
  };
}

function resolveNextQueryData<K extends QueryKey>(
  key: K,
  currentData: QueryDataByKey[K] | null,
  nextData: QueryDataByKey[K],
  request: ResolvedQueryRequest,
): QueryDataByKey[K] {
  if (
    key === "systemOverview" &&
    request.scope === "storage" &&
    currentData !== null
  ) {
    return {
      ...(currentData as SystemOverview),
      storage: (nextData as SystemOverview).storage,
    } as QueryDataByKey[K];
  }

  if (key === "distros" && currentData !== null) {
    return areDistroListsEqual(currentData as DistroInfo[], nextData as DistroInfo[])
      ? currentData
      : nextData;
  }

  if (key === "onlineDistros" && currentData !== null) {
    return areOnlineDistroListsEqual(
      currentData as OnlineDistro[],
      nextData as OnlineDistro[],
    )
      ? currentData
      : nextData;
  }

  return nextData;
}

function areListsEqual<T>(
  currentItems: T[],
  nextItems: T[],
  isEqual: (currentItem: T, nextItem: T) => boolean,
): boolean {
  return (
    currentItems.length === nextItems.length &&
    currentItems.every((currentItem, index) => {
      const nextItem = nextItems[index];
      return nextItem !== undefined && isEqual(currentItem, nextItem);
    })
  );
}

function areDistroListsEqual(
  currentDistros: DistroInfo[],
  nextDistros: DistroInfo[],
): boolean {
  return areListsEqual(currentDistros, nextDistros, (currentDistro, nextDistro) =>
    currentDistro.name === nextDistro.name &&
    getDistroStateKey(currentDistro.state) === getDistroStateKey(nextDistro.state) &&
    currentDistro.version === nextDistro.version &&
    currentDistro.is_default === nextDistro.is_default &&
    currentDistro.base_path === nextDistro.base_path &&
    currentDistro.vhd_file_name === nextDistro.vhd_file_name &&
    currentDistro.flavor === nextDistro.flavor &&
    currentDistro.os_version === nextDistro.os_version &&
    currentDistro.default_uid === nextDistro.default_uid,
  );
}

function getDistroStateKey(state: DistroInfo["state"]): string {
  return typeof state === "string" ? state : `Unknown:${state.Unknown}`;
}

function areOnlineDistroListsEqual(
  currentDistros: OnlineDistro[],
  nextDistros: OnlineDistro[],
): boolean {
  return areListsEqual(currentDistros, nextDistros, (currentDistro, nextDistro) =>
    currentDistro.name === nextDistro.name &&
    currentDistro.friendly_name === nextDistro.friendly_name,
  );
}
