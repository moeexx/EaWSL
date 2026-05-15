export const DEFAULT_MISSING_TEXT = "Not provided";
export const DEFAULT_LOADING_TEXT = "Loading";
const UNKNOWN_TEXT_ZH_CN = "\u672a\u77e5";

interface NormalizeOptionalTextOptions {
  treatUnknownAsMissing?: boolean;
}

interface FormatOptionalTextOptions extends NormalizeOptionalTextOptions {
  isPending?: boolean;
  loadingText?: string;
  missingText?: string;
}

export function normalizeOptionalText(
  value: string | null | undefined,
  options: NormalizeOptionalTextOptions = {},
): string | null {
  const trimmed = value?.trim();

  if (!trimmed || trimmed.length === 0) {
    return null;
  }

  if (
    options.treatUnknownAsMissing === true &&
    (/^unknown$/i.test(trimmed) || trimmed === UNKNOWN_TEXT_ZH_CN)
  ) {
    return null;
  }

  return trimmed;
}

export function formatOptionalText(
  value: string | null | undefined,
  options: FormatOptionalTextOptions = {},
): string {
  if (options.isPending === true) {
    return options.loadingText ?? DEFAULT_LOADING_TEXT;
  }

  return (
    normalizeOptionalText(value, {
      treatUnknownAsMissing: options.treatUnknownAsMissing,
    }) ??
    options.missingText ??
    DEFAULT_MISSING_TEXT
  );
}

export function formatBytes(
  value: number | null | undefined,
  fractionDigits = 2,
  missingText?: string,
): string {
  if (
    value === null ||
    value === undefined ||
    !Number.isFinite(value) ||
    value < 0
  ) {
    return missingText ?? DEFAULT_MISSING_TEXT;
  }

  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let size = value;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }

  return `${size.toFixed(fractionDigits)} ${units[unitIndex]}`;
}
