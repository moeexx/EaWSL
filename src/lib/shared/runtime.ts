import { getCopy, type AppLanguage } from "$lib/i18n";
import {
  getTauriCommandErrorMessage,
} from "$lib/tauri/errors";

export function hasTauriBridge(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function toErrorMessage(error: unknown): string {
  return (
    getExplicitErrorMessage(error) ?? getCopy().common.errors.operationFailed
  );
}

export function getExplicitErrorMessage(error: unknown): string | null {
  const tauriMessage = getTauriCommandErrorMessage(error);
  if (tauriMessage !== null && tauriMessage.length > 0) {
    return tauriMessage;
  }

  if (typeof error === "string" && error.length > 0) {
    return error;
  }

  if (error instanceof Error && error.message.length > 0) {
    return error.message;
  }

  return null;
}

export function createRequestId(): string {
  if (
    typeof crypto !== "undefined" &&
    typeof crypto.randomUUID === "function"
  ) {
    return crypto.randomUUID();
  }

  return `req-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

const fullTimeFormatterCache = new Map<AppLanguage, Intl.DateTimeFormat>();

export function getFullTimeFormatter(
  language: AppLanguage,
): Intl.DateTimeFormat {
  const cached = fullTimeFormatterCache.get(language);
  if (cached) {
    return cached;
  }

  const formatter = new Intl.DateTimeFormat(language, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
  });
  fullTimeFormatterCache.set(language, formatter);
  return formatter;
}

export function formatFullTime(value: Date, language: AppLanguage): string {
  return getFullTimeFormatter(language).format(value);
}

function pad(value: number, width: number): string {
  return String(value).padStart(width, "0");
}

export function formatLogTimestamp(value = new Date()): string {
  return `${value.getFullYear()}/${pad(value.getMonth() + 1, 2)}/${pad(
    value.getDate(),
    2,
  )} ${pad(value.getHours(), 2)}:${pad(value.getMinutes(), 2)}:${pad(
    value.getSeconds(),
    2,
  )}.${pad(value.getMilliseconds(), 3)}`;
}

export function formatLogPrefix(value = new Date()): string {
  return `[${formatLogTimestamp(value)}]`;
}

export type CacheLogKind =
  | "batch"
  | "start"
  | "hit"
  | "reuse"
  | "success"
  | "warn"
  | "error";

const CACHE_LOG_ICONS: Record<CacheLogKind, string> = {
  batch: "🚀",
  start: "🔍",
  hit: "📦",
  reuse: "🔁",
  success: "✅",
  warn: "⚠️",
  error: "❌",
};

export function formatLogTags(tags: readonly string[]): string {
  const normalized = tags.filter(
    (tag) => typeof tag === "string" && tag.length > 0,
  );

  if (normalized.length === 0) {
    return "";
  }

  return ` ${normalized.map((tag) => `[${tag}]`).join("")}`;
}

export function formatLogLinePrefix(
  kind: CacheLogKind,
  subject: string,
  tags: readonly string[] = [],
  value = new Date(),
): string {
  return `${formatLogPrefix(value)}${formatLogTags(tags)} [${
    CACHE_LOG_ICONS[kind]
  }][${subject}]`;
}
