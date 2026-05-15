import { get, writable } from "svelte/store";

import enUS, { type EnUsCopy } from "./locales/en-US";
import zhCN from "./locales/zh-CN";
import {
  SUPPORTED_LANGUAGES,
  type AppLanguage,
  type DeepPartial,
  type WidenCopy,
} from "./types";

const LANGUAGE_STORAGE_KEY = "eawsl.i18n.language";
const DEFAULT_LANGUAGE: AppLanguage = "en-US";

type CopyRecord = Record<string, unknown>;

export type AppCopy = WidenCopy<EnUsCopy>;

export interface I18nState {
  language: AppLanguage;
  copy: AppCopy;
}

const localeOverrides = {
  "en-US": enUS,
  "zh-CN": zhCN,
} satisfies Record<AppLanguage, DeepPartial<AppCopy>>;

const enUSCopy = enUS as AppCopy;

const store = writable<I18nState>(createState(readStoredLanguage()));

let started = false;

export const i18nState = {
  subscribe: store.subscribe,
};

export function startI18n(): void {
  if (started) {
    return;
  }

  started = true;
  store.set(createState(readStoredLanguage()));
}

export function getCopy(): AppCopy {
  return get(store).copy;
}

export function setLanguage(language: AppLanguage): void {
  store.set(createState(language));
  persistLanguage(language);
}

export function isAppLanguage(value: string): value is AppLanguage {
  return SUPPORTED_LANGUAGES.includes(value as AppLanguage);
}

function createState(language: AppLanguage): I18nState {
  return {
    language,
    copy: resolveCopy(language),
  };
}

export function resolveCopy(language: AppLanguage): AppCopy {
  return mergeWithFallback(enUSCopy, localeOverrides[language]);
}

function readStoredLanguage(): AppLanguage {
  if (typeof localStorage === "undefined") {
    return DEFAULT_LANGUAGE;
  }

  const stored = localStorage.getItem(LANGUAGE_STORAGE_KEY);
  return stored !== null && isAppLanguage(stored) ? stored : DEFAULT_LANGUAGE;
}

function persistLanguage(language: AppLanguage): void {
  if (typeof localStorage === "undefined") {
    return;
  }

  localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
}

function mergeWithFallback<T>(fallback: T, override: DeepPartial<T> | undefined): T {
  if (!isPlainRecord(fallback)) {
    return override === undefined ? fallback : (override as T);
  }

  const fallbackRecord = fallback as CopyRecord;
  const overrideRecord: CopyRecord = isPlainRecord(override) ? override : {};
  const merged: CopyRecord = {};

  for (const key of Object.keys(fallbackRecord)) {
    merged[key] = mergeWithFallback(
      fallbackRecord[key],
      overrideRecord[key] as DeepPartial<unknown> | undefined,
    );
  }

  return merged as T;
}

function isPlainRecord(value: unknown): value is CopyRecord {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value)
  );
}
