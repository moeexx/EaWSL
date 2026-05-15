import type { AppCopy } from "$lib/i18n";

import type { QueryDataByKey, QueryEntry, QueryKey } from "./types";

export function getQueryErrorMessage<K extends QueryKey>(
  key: K,
  entry: QueryEntry<QueryDataByKey[K]>,
  copy: AppCopy,
): string {
  return entry.errorMessage ?? copy.queryCache.errors[key];
}

export function getQueryRecoveringHint<K extends QueryKey>(
  key: K,
  entry: QueryEntry<QueryDataByKey[K]>,
  copy: AppCopy,
): string | null {
  if (!entry.isRecovering || entry.recoveringState === null) {
    return null;
  }

  const subject =
    entry.recoveringState.code === "host-command-timed-out"
      ? copy.queryCache.recovering.subjects.hostCommandTimedOut
      : copy.queryCache.recovering.subjects[key];

  return entry.recoveringState.hasSuccessfulData
    ? copy.queryCache.recovering.withFallbackData(subject)
    : copy.queryCache.recovering.withoutFallbackData(subject);
}
