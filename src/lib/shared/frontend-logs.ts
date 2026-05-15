import { formatLogLinePrefix, type CacheLogKind } from "./runtime";
import type { RefreshReason } from "./refresh-reasons";

export type FrontendFlowReason = Extract<
  RefreshReason,
  "startup" | "page-enter" | "manual"
>;

export type FrontendLogStage = "foreground" | "background";

function logLine(
  method: "info" | "warn",
  kind: CacheLogKind,
  subject: string,
  tags: readonly string[],
  message: string,
  payload?: unknown,
): void {
  const line = `${formatLogLinePrefix(kind, subject, tags)} ${message}`;

  if (payload === undefined) {
    console[method](line);
    return;
  }

  console[method](line, payload);
}

function getReasonTag(reason: RefreshReason): string {
  const labels: Record<RefreshReason, string> = {
    startup: "Startup",
    "page-enter": "Page enter",
    manual: "Manual",
    "action-sync": "Action sync",
    background: "Background refresh",
  };

  return labels[reason];
}

function getFlowStartMessage(reason: FrontendFlowReason): string {
  if (reason === "startup") {
    return "App startup begins cache warmup.";
  }

  if (reason === "page-enter") {
    return "Page entry begins refresh.";
  }

  return "Manual action begins refresh.";
}

function getFlowTags(reason: FrontendFlowReason): string[] {
  return [getReasonTag(reason), getStageTag("foreground")];
}

function getStageTag(stage: FrontendLogStage): string {
  return stage === "foreground" ? "Foreground" : "Background";
}

function getStepTags(
  reason: RefreshReason,
  stage: FrontendLogStage,
  ...extra: string[]
): string[] {
  return [getReasonTag(reason), getStageTag(stage), ...extra];
}

export async function runLoggedRefreshFlow<T>(
  subject: string,
  reason: FrontendFlowReason,
  run: () => Promise<T>,
): Promise<T> {
  logLine(
    "info",
    "start",
    subject,
    getFlowTags(reason),
    getFlowStartMessage(reason),
  );

  try {
    return await run();
  } finally {
    logLine(
      "info",
      "batch",
      subject,
      getFlowTags(reason),
      "====== Complete ======",
    );
  }
}

export function logCacheReadStart(
  subject: string,
  reason: RefreshReason,
  stage: FrontendLogStage,
): void {
  logLine(
    "info",
    "start",
    subject,
    getStepTags(reason, stage, "Read cache"),
    "Checking cache.",
  );
}

export function logCacheReadHit(
  subject: string,
  reason: RefreshReason,
  stage: FrontendLogStage,
  payload: unknown,
): void {
  logLine(
    "info",
    "hit",
    subject,
    getStepTags(reason, stage, "Read cache"),
    "Cache hit.",
    payload,
  );
}

export function logBackendRequestStart(
  subject: string,
  reason: RefreshReason,
  stage: FrontendLogStage,
): void {
  logLine(
    "info",
    "start",
    subject,
    getStepTags(reason, stage, "Backend request", "Write cache"),
    "Starting new request.",
  );
}

export function logBackendRequestReuse(
  subject: string,
  reason: RefreshReason,
  stage: FrontendLogStage,
): void {
  logLine(
    "info",
    "reuse",
    subject,
    getStepTags(reason, stage, "Backend request", "Write cache"),
    "Reusing in-flight request.",
  );
}

export function logBackendRequestSuccess(
  subject: string,
  reason: RefreshReason,
  stage: FrontendLogStage,
  payload: unknown,
): void {
  logLine(
    "info",
    "success",
    subject,
    getStepTags(reason, stage, "Backend request", "Write cache"),
    "Request completed and cache was written.",
    payload,
  );
}

export function logBackendRequestRecovering(
  subject: string,
  reason: RefreshReason,
  stage: FrontendLogStage,
  payload: unknown,
): void {
  logLine(
    "warn",
    "warn",
    subject,
    getStepTags(reason, stage, "Backend request", "Write cache"),
    "Request returned a recoverable error; keeping the last successful result.",
    payload,
  );
}

export function logBackendRequestError(
  subject: string,
  reason: RefreshReason,
  stage: FrontendLogStage,
  payload: unknown,
): void {
  logLine(
    "warn",
    "error",
    subject,
    getStepTags(reason, stage, "Backend request", "Write cache"),
    "Request failed.",
    payload,
  );
}

export function logRefreshWait(
  subject: string,
  reason: RefreshReason,
  stage: FrontendLogStage,
  message: string,
): void {
  logLine(
    "info",
    "start",
    subject,
    getStepTags(reason, stage, "Wait"),
    message,
  );
}

export function logProbeRead(
  subject: string,
  tags: readonly string[],
  message: string,
  payload?: unknown,
): void {
  logLine("info", "start", subject, tags, message, payload);
}

export function logProbeWrite(
  subject: string,
  tags: readonly string[],
  message: string,
  payload?: unknown,
): void {
  logLine("info", "success", subject, tags, message, payload);
}
