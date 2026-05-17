import { findDefaultDistro } from "$lib/shared/distros";
import { formatOptionalText } from "$lib/shared/format";

import type { QueryCacheState } from "$lib/query-cache";
import type { AppCopy } from "$lib/i18n";

export type InfoItem = {
  label: string;
  value: string;
};

type DistroList = NonNullable<QueryCacheState["distros"]["data"]>;
type WslVersionData = NonNullable<QueryCacheState["wslVersion"]["data"]>;
type WslVersionKey = Extract<
  keyof WslVersionData,
  "wsl" | "kernel" | "wslg" | "msrdc" | "direct3d" | "dxcore"
>;

const wslVersionFields: Array<{
  labelKey: keyof AppCopy["overview"]["wsl"]["fields"];
  key: WslVersionKey;
}> = [
  { labelKey: "wsl", key: "wsl" },
  { labelKey: "kernel", key: "kernel" },
  { labelKey: "wslg", key: "wslg" },
  { labelKey: "msrdc", key: "msrdc" },
  { labelKey: "direct3d", key: "direct3d" },
  { labelKey: "dxcore", key: "dxcore" },
];

export function buildWslInfoItems(
  state: QueryCacheState,
  copy: AppCopy,
): InfoItem[] {
  const wslVersion = state.wslVersion.data;
  const hasError = state.wslVersion.hasError;
  const isLoading = state.wslVersion.activity === "loading";
  const isRecovering = state.wslVersion.isRecovering;

  return [
    ...wslVersionFields.map(({ labelKey, key }) => ({
      label: copy.overview.wsl.fields[labelKey],
      value: formatWslValue(
        wslVersion?.[key],
        hasError,
        isLoading,
        isRecovering,
        copy,
      ),
    })),
    {
      label: copy.overview.wsl.fields.defaultDistro,
      value: getDefaultDistroLabel(state, copy),
    },
    {
      label: copy.overview.wsl.fields.distroCount,
      value: getDistroCountLabel(state, copy),
    },
  ];
}

function hasValue<T extends string | number>(
  value: T | null | undefined,
): value is T {
  return value !== null && value !== undefined && value !== "";
}

function formatText(
  value: string | null | undefined,
  isPending: boolean,
  copy: AppCopy,
): string {
  return formatOptionalText(value, {
    isPending,
    loadingText: copy.common.loading,
    missingText: copy.common.missing,
  });
}

function formatWslValue(
  value: string | null | undefined,
  hasError: boolean,
  isPending: boolean,
  isRecovering: boolean,
  copy: AppCopy,
): string {
  const valueExists = hasValue(value);

  if (isPending && !valueExists) {
    return copy.common.loading;
  }

  if (hasError && !valueExists) {
    return copy.common.readFailed;
  }

  if (isRecovering && !valueExists) {
    return copy.overview.wsl.status.recovering;
  }

  return formatText(value, false, copy);
}

function formatDistroState(
  state: QueryCacheState,
  getValue: (distros: DistroList) => string,
  copy: AppCopy,
): string {
  const distros = (state.distros.data ?? []) as DistroList;
  const hasDistros = distros.length > 0;

  if (state.distros.activity === "loading" && !hasDistros) {
    return copy.common.loading;
  }

  if (state.distros.hasError && !hasDistros) {
    return copy.common.readFailed;
  }

  if (state.distros.isRecovering && !hasDistros) {
    return copy.overview.wsl.status.recovering;
  }

  return getValue(distros);
}

function getDefaultDistroLabel(state: QueryCacheState, copy: AppCopy): string {
  return formatDistroState(
    state,
    (distros) =>
      findDefaultDistro(distros)?.name ?? copy.overview.wsl.status.notSet,
    copy,
  );
}

function getDistroCountLabel(state: QueryCacheState, copy: AppCopy): string {
  return formatDistroState(state, (distros) => String(distros.length), copy);
}
