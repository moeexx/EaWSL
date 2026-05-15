import { formatBytes, normalizeOptionalText } from "$lib/shared/format";
import type { AppCopy } from "$lib/i18n";
import type { VhdSizeEntry } from "$lib/probes/distro-vhd-size";
import type { DistroState } from "$lib/tauri/wsl";

import type {
  DistroWorkspaceActionButtonKey,
  DistroStateBadge,
  ShutdownMode,
} from "./distro-workspace-types";

const knownStateBadgeClasses: Record<
  Extract<DistroState, string>,
  Pick<DistroStateBadge, "dotClass" | "textClass">
> = {
  Running: {
    textClass: "text-emerald-700",
    dotClass: "bg-emerald-500",
  },
  Stopped: {
    textClass: "text-shell-600",
    dotClass: "bg-shell-400",
  },
  Installing: {
    textClass: "text-amber-700",
    dotClass: "bg-amber-500",
  },
};

export function getStateBadge(
  state: DistroState,
  copy: AppCopy,
): DistroStateBadge {
  if (typeof state === "string" && state in knownStateBadgeClasses) {
    const stateLabel =
      state === "Running"
        ? copy.common.running
        : state === "Stopped"
          ? copy.common.stopped
          : copy.common.installing;
    return {
      label: stateLabel,
      ...knownStateBadgeClasses[state],
    };
  }

  const unknownState =
    typeof state === "object" && "Unknown" in state ? state.Unknown : String(state);

  return {
    label: copy.distros.row.unknownState(unknownState),
    textClass: "text-violet-700",
    dotClass: "bg-violet-500",
  };
}

export function formatDefaultUser(
  value: number | null | undefined,
  copy: AppCopy,
): string {
  return value === null || value === undefined
    ? copy.common.missing
    : copy.distros.row.defaultUser.value(value);
}

export function formatFlavorVersion(
  flavor: string | null | undefined,
  osVersion: string | null | undefined,
): string | null {
  const segments = [
    normalizeOptionalText(flavor, { treatUnknownAsMissing: true }),
    normalizeOptionalText(osVersion, { treatUnknownAsMissing: true }),
  ].filter((segment): segment is string => segment !== null);

  return segments.length > 0 ? segments.join(" ") : null;
}

export function isCompactFlavorVersion(value: string | null): boolean {
  return value !== null && value.length <= 26;
}

export function getVhdSizeLabel(
  entry: VhdSizeEntry,
  expanded: boolean,
  copy: AppCopy,
): string {
  if (expanded && (entry.status === "idle" || entry.status === "loading")) {
    return copy.distros.row.probing;
  }

  if (entry.status === "ready") {
    return formatBytes(entry.bytes, 2, copy.common.missing);
  }
  if (entry.status === "missing") return copy.common.missing;
  if (entry.status === "error") return copy.common.readFailed;
  return copy.distros.row.unread;
}

export function getExpandLabel(expanded: boolean, copy: AppCopy): string {
  return expanded ? copy.distros.row.collapse : copy.distros.row.expand;
}

export function getShutdownButtonLabel(
  activeActionButtonKey: DistroWorkspaceActionButtonKey,
  copy: AppCopy,
): string {
  return activeActionButtonKey === "shutdown-all"
    ? copy.common.stopping
    : copy.distros.buttons.shutdownWsl;
}

export function getShutdownButtonRefreshingLabel(
  shutdownMode: ShutdownMode,
  copy: AppCopy,
): string {
  return shutdownMode === "force"
    ? copy.distros.buttons.forceStopping
    : copy.common.stopping;
}
