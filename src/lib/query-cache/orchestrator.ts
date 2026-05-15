import { getSystemOverview } from "$lib/tauri/system";
import {
  getWslVersion,
  listDistros,
  listOnlineDistros,
} from "$lib/tauri/wsl";
import { isRecoverableCommandError } from "$lib/tauri/errors";
import {
  DEFAULT_BACKGROUND_REFRESH_SETTINGS,
  getBackgroundRefreshSettings,
  type BackgroundRefreshSettings,
  type BackgroundRefreshTarget,
} from "$lib/tauri/settings";
import {
  logBackendRequestError,
  logBackendRequestRecovering,
  logBackendRequestReuse,
  logBackendRequestStart,
  logBackendRequestSuccess,
  logCacheReadHit,
  logCacheReadStart,
  logRefreshWait,
  runLoggedRefreshFlow,
  type FrontendLogStage,
} from "$lib/shared/frontend-logs";

import type {
  QueryDataByKey,
  QueryKey,
  QueryRefreshPlan,
  QueryRefreshReason,
  QueryRefreshResult,
  QueryRefreshResults,
  QueryRequest,
} from "./types";
import {
  getFreshSinceResult,
  markQueryStarted,
  readQueryEntry,
  resolveEffectiveQueryRequest,
  shouldRequest,
  writeQueryFailure,
  writeQueryRecovering,
  writeQuerySuccess,
  type ResolvedQueryRequest,
} from "./state";

const REFRESH_WAIT_SUBJECT = "Refresh orchestration";

const FOREGROUND_QUERY_INTERVAL_MS = 50;
const FOREGROUND_TO_BACKGROUND_DELAY_MS = 100;
const BACKGROUND_QUERY_INTERVAL_MS = 100;
const MILLISECONDS_PER_MINUTE = 60 * 1000;

type InFlightQuery = {
  request: Promise<QueryRefreshResult<unknown>>;
};

const queries: {
  [K in Exclude<QueryKey, "systemOverview">]: () => Promise<QueryDataByKey[K]>;
} = {
  distros: listDistros,
  wslVersion: getWslVersion,
  onlineDistros: listOnlineDistros,
};

const inFlightRequests: Record<string, InFlightQuery | undefined> = {};

let started = false;
let refreshInterval: ReturnType<typeof setInterval> | null = null;
let backgroundRefreshSettings = DEFAULT_BACKGROUND_REFRESH_SETTINGS;
let backgroundRefreshConfigurationVersion = 0;

export async function refreshQueries(
  plan: QueryRefreshPlan,
): Promise<QueryRefreshResults> {
  const planStartedAt = Date.now();
  const foregroundSequence = runQuerySequence(
    plan.foreground,
    plan.reason,
    FOREGROUND_QUERY_INTERVAL_MS,
    planStartedAt,
    "foreground",
    plan.reason,
  );

  const results = await foregroundSequence;

  if (plan.foreground.length > 0) {
    await waitForForegroundMinDuration(
      planStartedAt,
      plan.foregroundMinDurationMs ?? 0,
      plan.reason,
    );
  }

  if (plan.background.length > 0) {
    if (plan.foreground.length > 0) {
      logRefreshWait(
        REFRESH_WAIT_SUBJECT,
        plan.reason,
        "background",
        `Foreground to background wait: ${FOREGROUND_TO_BACKGROUND_DELAY_MS} ms.`,
      );
      await delay(FOREGROUND_TO_BACKGROUND_DELAY_MS);
    }

    const backgroundResults = await runQuerySequence(
      plan.background,
      getBackgroundReason(plan.reason),
      BACKGROUND_QUERY_INTERVAL_MS,
      planStartedAt,
      "background",
      plan.reason,
    ).catch((error) => {
      logBackendRequestError(
        "Background refresh sequence",
        plan.reason,
        "background",
        error,
      );
      return {};
    });

    return {
      ...results,
      ...backgroundResults,
    };
  }

  return results;
}

export function configureQueryCacheBackgroundRefresh(
  settings: BackgroundRefreshSettings,
): void {
  backgroundRefreshConfigurationVersion += 1;
  applyBackgroundRefreshSettings(settings);
}

export function startQueryCache(): void {
  if (started) {
    return;
  }

  started = true;

  void runLoggedRefreshFlow("Global cache", "startup", () =>
    refreshQueries({
      foreground: ["distros"],
      background: [
        { key: "systemOverview", scope: "full" },
        "wslVersion",
        "onlineDistros",
      ],
      reason: "startup",
    }),
  );

  void loadBackgroundRefreshSettings();
}

export function stopQueryCache(): void {
  clearBackgroundRefreshInterval();
  started = false;
}

async function loadBackgroundRefreshSettings(): Promise<void> {
  const configurationVersion = backgroundRefreshConfigurationVersion;
  const settings = await getBackgroundRefreshSettings();

  if (!started || configurationVersion !== backgroundRefreshConfigurationVersion) {
    return;
  }

  applyBackgroundRefreshSettings(settings);
}

function applyBackgroundRefreshSettings(settings: BackgroundRefreshSettings): void {
  backgroundRefreshSettings = settings;
  restartBackgroundRefreshInterval();
}

function restartBackgroundRefreshInterval(): void {
  clearBackgroundRefreshInterval();

  if (!started) {
    return;
  }

  refreshInterval = setInterval(() => {
    void refreshQueries({
      foreground: [],
      background: buildBackgroundRefreshRequests(backgroundRefreshSettings.targets),
      reason: "background",
    });
  }, backgroundRefreshSettings.intervalMinutes * MILLISECONDS_PER_MINUTE);
}

function clearBackgroundRefreshInterval(): void {
  if (refreshInterval === null) {
    return;
  }

  clearInterval(refreshInterval);
  refreshInterval = null;
}

async function runQuerySequence(
  requests: readonly QueryRequest[],
  reason: QueryRefreshReason,
  intervalMs: number,
  planStartedAt: number,
  stage: FrontendLogStage,
  sourceReason: QueryRefreshReason,
): Promise<QueryRefreshResults> {
  const results: QueryRefreshResults = {};

  for (const [index, request] of requests.entries()) {
    const resolvedRequest = resolveQueryRequest(request);
    const result = await refreshQuery(resolvedRequest, reason, {
      skipFreshSince: shouldSkipFreshSince(reason) ? planStartedAt : null,
      stage,
      sourceReason,
    });

    writePlanResult(results, resolvedRequest.key, result);

    if (index < requests.length - 1) {
      logRefreshWait(
        REFRESH_WAIT_SUBJECT,
        sourceReason,
        stage,
        `${stage === "foreground" ? "Foreground" : "Background"} request interval: ${intervalMs} ms.`,
      );
      await delay(intervalMs);
    }
  }

  return results;
}

async function refreshQuery<K extends QueryKey>(
  request: ResolvedQueryRequest,
  reason: QueryRefreshReason,
  options: {
    skipFreshSince?: number | null;
    stage?: FrontendLogStage;
    sourceReason?: QueryRefreshReason;
  } = {},
): Promise<QueryRefreshResult<QueryDataByKey[K]>> {
  const key = request.key as K;
  const entry = readQueryEntry(key);
  const effectiveRequest = resolveEffectiveQueryRequest(request, entry);
  const freshSinceResult = getFreshSinceResult(
    entry,
    options.skipFreshSince,
    effectiveRequest,
  );
  const stage = options.stage ?? "foreground";
  const sourceReason = options.sourceReason ?? reason;
  const subject = getQuerySubject(effectiveRequest);

  if (freshSinceResult) {
    logCacheReadStart(subject, sourceReason, stage);
    logCacheReadHit(subject, sourceReason, stage, freshSinceResult.data);
    return freshSinceResult;
  }

  if (!shouldRequest(entry, reason, effectiveRequest)) {
    const result = freshResult(entry.data as QueryDataByKey[K]);
    logCacheReadStart(subject, sourceReason, stage);
    logCacheReadHit(subject, sourceReason, stage, result.data);
    return result;
  }

  const inFlightKey = getInFlightKey(effectiveRequest);
  const existingRequest = inFlightRequests[inFlightKey];

  if (existingRequest) {
    markQueryStarted(key, reason);
    logBackendRequestReuse(subject, sourceReason, stage);

    const existingResult = (await existingRequest.request) as QueryRefreshResult<
      QueryDataByKey[K]
    >;

    if (reason === "action-sync" && key === "distros") {
      return refreshQuery(effectiveRequest, reason, options);
    }

    if (existingResult.kind === "fresh") {
      return existingResult;
    }

    const latestFreshSinceResult = getFreshSinceResult(
      readQueryEntry(key),
      options.skipFreshSince,
      effectiveRequest,
    );

    if (latestFreshSinceResult) {
      return latestFreshSinceResult;
    }

    if (reason === "manual") {
      return refreshQuery(effectiveRequest, reason, options);
    }

    return existingResult;
  }

  markQueryStarted(key, reason);
  logBackendRequestStart(subject, sourceReason, stage);

  const requestPromise = executeQueryRequest(effectiveRequest)
    .then((data) => {
      const result = writeQuerySuccess(
        key,
        data as QueryDataByKey[K],
        effectiveRequest,
      );
      logBackendRequestSuccess(subject, sourceReason, stage, result.data);
      return result;
    })
    .catch((error) => {
      if (isRecoverableCommandError(error)) {
        const recovering = writeQueryRecovering(key, error, reason);
        logBackendRequestRecovering(subject, sourceReason, stage, {
          error,
          fallbackData: recovering.fallbackData,
        });
        return recovering.result;
      }

      const result = writeQueryFailure(key, error, reason);
      logBackendRequestError(subject, sourceReason, stage, error);
      return result;
    })
    .finally(() => {
      if (inFlightRequests[inFlightKey]?.request === requestPromise) {
        delete inFlightRequests[inFlightKey];
      }
    });

  inFlightRequests[inFlightKey] = {
    request: requestPromise as Promise<QueryRefreshResult<unknown>>,
  };

  return requestPromise;
}

function shouldSkipFreshSince(reason: QueryRefreshReason): boolean {
  return reason === "manual" || reason === "background";
}

function freshResult<T>(data: T): QueryRefreshResult<T> {
  return {
    kind: "fresh",
    data,
  };
}

function writePlanResult<K extends QueryKey>(
  results: QueryRefreshResults,
  key: K,
  result: QueryRefreshResult<QueryDataByKey[K]>,
): void {
  results[key] = result as QueryRefreshResults[K];
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

async function waitForForegroundMinDuration(
  startedAt: number,
  minDurationMs: number,
  reason: QueryRefreshReason,
): Promise<void> {
  if (minDurationMs <= 0) {
    return;
  }

  const elapsedMs = Date.now() - startedAt;
  if (elapsedMs >= minDurationMs) {
    return;
  }

  const waitMs = minDurationMs - elapsedMs;
  logRefreshWait(
    REFRESH_WAIT_SUBJECT,
    reason,
    "foreground",
    `Foreground minimum wait: filling ${waitMs} ms.`,
  );
  await delay(waitMs);
}

function getBackgroundReason(reason: QueryRefreshReason): QueryRefreshReason {
  return reason === "manual" || reason === "action-sync" ? "background" : reason;
}

function resolveQueryRequest(request: QueryRequest): ResolvedQueryRequest {
  if (typeof request === "string") {
    return {
      key: request,
      scope: request === "systemOverview" ? "full" : null,
    };
  }

  return {
    key: request.key,
    scope: request.scope,
  };
}

async function executeQueryRequest<K extends QueryKey>(
  request: ResolvedQueryRequest,
): Promise<QueryDataByKey[K]> {
  if (request.key === "systemOverview") {
    return (await getSystemOverview({
      scope: request.scope ?? "full",
    })) as QueryDataByKey[K];
  }

  return queries[request.key as Exclude<QueryKey, "systemOverview">]() as Promise<
    QueryDataByKey[K]
  >;
}

function getInFlightKey(request: ResolvedQueryRequest): string {
  if (request.key === "systemOverview") {
    return `${request.key}:${request.scope ?? "full"}`;
  }

  return request.key;
}

function getQuerySubject(request: ResolvedQueryRequest): string {
  if (request.key === "systemOverview") {
    return request.scope === "storage"
      ? "System overview (storage)"
      : "System overview (full)";
  }

  const labels: Record<Exclude<QueryKey, "systemOverview">, string> = {
    distros: "Distro list",
    wslVersion: "WSL version",
    onlineDistros: "Online distro list",
  };

  return labels[request.key as Exclude<QueryKey, "systemOverview">];
}

function buildBackgroundRefreshRequests(
  targets: readonly BackgroundRefreshTarget[],
): QueryRequest[] {
  return targets.map((target) => {
    if (target === "systemOverviewStorage") {
      return { key: "systemOverview", scope: "storage" };
    }

    return target;
  });
}
