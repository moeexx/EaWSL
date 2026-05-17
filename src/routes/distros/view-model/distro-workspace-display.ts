import {
  hasHiddenRowActions,
  sortUiActionHiddenDistrosLast,
} from "$lib/shared/distros";
import { getDistroLogoSrc } from "$lib/shared/distro-logos";
import type {
  VhdSizeCacheState,
  VhdSizeEntry,
} from "$lib/probes/distro-vhd-size";
import {
  getQueryErrorMessage,
  getQueryRecoveringHint,
  type QueryCacheState,
} from "$lib/query-cache";
import type { AppCopy } from "$lib/i18n";
import type { DistroInfo } from "$lib/tauri/wsl";

import {
  createDistroRowScope,
  type DistroWorkspaceActionButtonKey,
  type DistroRowView,
  type DistroWorkspaceNotice,
  type DistroWorkspaceOverlayScope,
  type DistroWorkspaceOverlayState,
  type DistroWorkspaceSectionState,
  type DistroWorkspaceView,
  type ShutdownMode,
} from "./distro-workspace-types";
import {
  isBlockingActionOverlayPhase,
  isSameActionOverlayScope,
} from "./action-overlay";
import {
  formatDefaultUser,
  formatFlavorVersion,
  getExpandLabel,
  getShutdownButtonLabel,
  getShutdownButtonRefreshingLabel,
  getStateBadge,
  getVhdSizeLabel,
  isCompactFlavorVersion,
} from "./distro-workspace-display-format";
import { buildDistroExportFormats } from "./distro-export-rules";

interface DistroWorkspaceViewInput {
  copy: AppCopy;
  queryState: QueryCacheState;
  vhdSizeState: VhdSizeCacheState;
  workspaceRefreshing: boolean;
  activeActionButtonKey: DistroWorkspaceActionButtonKey;
  expandedPanels: Record<string, boolean>;
  actionOverlays: DistroWorkspaceOverlayState[];
  shutdownMode: ShutdownMode;
  hasActiveLongTask: boolean;
  getVhdSizeEntry: (
    state: VhdSizeCacheState,
    distro: DistroInfo,
  ) => VhdSizeEntry;
  getInstallLocation: (distro: DistroInfo) => string | null;
}

export function buildDistroWorkspaceView(
  input: DistroWorkspaceViewInput,
): DistroWorkspaceView {
  const distros = input.queryState.distros.data ?? [];
  const refreshActivity = input.queryState.distros.activity;
  const refreshDisabled =
    input.workspaceRefreshing ||
    refreshActivity === "loading" ||
    input.activeActionButtonKey !== null;
  const actionButtonsDisabled =
    refreshDisabled ||
    input.actionOverlays.some((overlay) =>
      isBlockingActionOverlayPhase(overlay.phase),
    );
  const rows = sortUiActionHiddenDistrosLast(distros).map((distro) =>
    buildDistroRowView(input, distro, actionButtonsDisabled),
  );

  return {
    notices: buildWorkspaceNotices(input),
    refreshButton: {
      label: input.copy.common.refreshList,
      refreshingLabel: input.copy.common.refreshing,
      refreshing: input.workspaceRefreshing,
      disabled: refreshDisabled,
    },
    shutdownButton: {
      label: getShutdownButtonLabel(input.activeActionButtonKey, input.copy),
      refreshingLabel: getShutdownButtonRefreshingLabel(
        input.shutdownMode,
        input.copy,
      ),
      running: input.activeActionButtonKey === "shutdown-all",
      disabled: actionButtonsDisabled || input.hasActiveLongTask,
    },
    section: {
      title: input.copy.distros.section.title,
      state: getSectionState(input.queryState),
      count: distros.length,
      ...getSectionEmptyState(input.queryState, input.copy),
      rows,
    },
  };
}

export function getWorkspaceActionOverlay(
  overlays: DistroWorkspaceOverlayState[],
  scope: DistroWorkspaceOverlayScope,
): DistroWorkspaceOverlayState | null {
  return (
    overlays.find((overlay) =>
      isSameActionOverlayScope(overlay.scope, scope),
    ) ?? null
  );
}

export function getDistroRefreshFailureMessage(
  state: QueryCacheState,
  copy: AppCopy,
): string {
  return getQueryErrorMessage("distros", state.distros, copy);
}

export function getDistroRecoveringMessage(
  state: QueryCacheState,
  copy: AppCopy,
): string {
  return (
    getQueryRecoveringHint("distros", state.distros, copy) ??
    copy.distros.messages.recovering
  );
}

function buildWorkspaceNotices(
  input: DistroWorkspaceViewInput,
): DistroWorkspaceNotice[] {
  const notices: DistroWorkspaceNotice[] = [];

  if (input.queryState.distros.hasError) {
    notices.push({
      key: "distros-error",
      tone: "error",
      title: input.copy.distros.notices.listReadFailed,
      message: getDistroRefreshFailureMessage(input.queryState, input.copy),
    });
  }

  if (input.queryState.distros.isRecovering) {
    notices.push({
      key: "distros-recovering",
      tone: "warning",
      title: input.copy.distros.notices.listRecovering,
      message: getDistroRecoveringMessage(input.queryState, input.copy),
    });
  }

  return notices;
}

function buildDistroRowView(
  input: DistroWorkspaceViewInput,
  distro: DistroInfo,
  actionsDisabled: boolean,
): DistroRowView {
  const hiddenRowActions = hasHiddenRowActions(distro);
  const expanded = input.expandedPanels[distro.name] === true;
  const vhdSizeEntry = input.getVhdSizeEntry(input.vhdSizeState, distro);
  const unregisterOverlay = getWorkspaceActionOverlay(
    input.actionOverlays,
    createDistroRowScope(distro.name, "unregister"),
  );
  const flavorVersion = formatFlavorVersion(distro.flavor, distro.os_version);

  return {
    name: distro.name,
    logoSrc: getDistroLogoSrc(distro.flavor, distro.name),
    panelId: getDistroPanelId(distro.name),
    state: getStateBadge(distro.state, input.copy),
    isDefault: distro.is_default,
    isProtected: hiddenRowActions,
    protectedMessage: hiddenRowActions
      ? input.copy.distros.row.protectedMessage
      : null,
    flavorVersion,
    flavorVersionCompact: isCompactFlavorVersion(flavorVersion),
    versionLabel: `WSL ${distro.version}`,
    actionsDisabled,
    terminateRunning:
      input.activeActionButtonKey === `terminate:${distro.name}`,
    unregisterBusy:
      unregisterOverlay?.phase === "running" ||
      unregisterOverlay?.phase === "verifying",
    deleteLabel:
      unregisterOverlay?.phase === "verifying"
        ? input.copy.distros.row.deleteVerifying
        : unregisterOverlay?.phase === "running"
          ? input.copy.distros.row.deleting
          : input.copy.distros.row.delete,
    settingDefault:
      input.activeActionButtonKey === `set-default:${distro.name}`,
    expanded,
    expandLabel: getExpandLabel(expanded, input.copy),
    exportLabel: input.copy.common.export,
    details: [
      {
        key: "default-user",
        label: input.copy.distros.row.defaultUser.label,
        value: formatDefaultUser(distro.default_uid, input.copy),
        variant: "strong",
      },
      {
        key: "vhd-size",
        label: input.copy.distros.row.details.vhdSize,
        value: getVhdSizeLabel(vhdSizeEntry, expanded, input.copy),
        variant: "strong",
      },
      {
        key: "install-location",
        label: input.copy.distros.row.details.installLocation,
        value: input.getInstallLocation(distro) ?? input.copy.common.missing,
        variant: "mono",
      },
    ],
    exportMenu: {
      title: input.copy.distros.row.exportMenu.title,
      fileNameLabel: input.copy.distros.row.exportMenu.fileNameLabel,
      fileNamePlaceholder:
        input.copy.distros.row.exportMenu.fileNamePlaceholder,
      formatLabel: input.copy.distros.row.exportMenu.formatLabel,
      directoryLabel: input.copy.distros.row.exportMenu.directoryLabel,
      directoryPlaceholder:
        input.copy.distros.row.exportMenu.directoryPlaceholder,
      chooseDirectory: input.copy.common.chooseDirectory,
      submit: input.copy.distros.row.exportMenu.submit,
      exporting: input.copy.distros.row.exportMenu.exporting,
      defaultFileName: distro.name,
      defaultDirectory: "",
      formats: buildDistroExportFormats(input.copy),
      errors: {
        fileNameRequired:
          input.copy.distros.row.exportMenu.validation.fileNameRequired,
        fileNameInvalid:
          input.copy.distros.row.exportMenu.validation.fileNameInvalid,
        fileNameSuffixNotAllowed:
          input.copy.distros.row.exportMenu.validation.fileNameSuffixNotAllowed,
        directoryRequired:
          input.copy.distros.row.exportMenu.validation.directoryRequired,
        noTauriDirectoryPicker:
          input.copy.distros.row.exportMenu.validation.noTauriDirectoryPicker,
      },
    },
  };
}

function getDistroPanelId(distroName: string): string {
  const encodedName = Array.from(
    distroName,
    (char) => char.codePointAt(0)?.toString(36) ?? "0",
  ).join("-");

  return `distro-advanced-${encodedName || "unknown"}`;
}

function getSectionState(state: QueryCacheState): DistroWorkspaceSectionState {
  const distros = state.distros.data ?? [];

  if (state.distros.activity === "loading") return "loading";
  if (state.distros.isRecovering && distros.length === 0) return "recovering";
  return distros.length === 0 ? "empty" : "ready";
}

function getSectionEmptyState(
  state: QueryCacheState,
  copy: AppCopy,
): {
  emptyTitle: string | null;
  emptyMessage: string | null;
} {
  const sectionState = getSectionState(state);

  if (sectionState === "loading") {
    return {
      emptyTitle: copy.distros.section.loadingTitle,
      emptyMessage: copy.distros.section.loadingMessage,
    };
  }

  if (sectionState === "recovering") {
    return {
      emptyTitle: copy.distros.section.recoveringTitle,
      emptyMessage: getDistroRecoveringMessage(state, copy),
    };
  }

  if (sectionState === "empty") {
    return {
      emptyTitle: copy.distros.section.emptyTitle,
      emptyMessage: copy.distros.section.emptyMessage,
    };
  }

  return { emptyTitle: null, emptyMessage: null };
}
