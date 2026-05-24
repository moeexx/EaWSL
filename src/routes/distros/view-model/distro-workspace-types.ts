import type { ExportFormat } from "$lib/tauri/wsl";

import type { ActionOverlayScope, ActionOverlayState } from "./action-overlay";

export type DistroWorkspaceOverlayScope =
  | Extract<ActionOverlayScope, { kind: "distros-list" }>
  | Extract<ActionOverlayScope, { kind: "distro-row" }>;

export type DistroWorkspaceOverlayState = Omit<ActionOverlayState, "scope"> & {
  scope: DistroWorkspaceOverlayScope;
};

export type DistroWorkspaceNoticeTone =
  | "error"
  | "info"
  | "success"
  | "warning";

export type ShutdownMode = "normal" | "force" | null;
export type DistroWorkspaceActionButtonKey = string | null;

export interface DistroWorkspaceNotice {
  key: string;
  tone: DistroWorkspaceNoticeTone;
  title: string;
  message: string;
}

export interface DistroStateBadge {
  label: string;
  textClass: string;
  dotClass: string;
}

export interface DistroDetailItem {
  key: string;
  label: string;
  value: string;
  variant: "normal" | "strong" | "mono";
}

export interface DistroExportFormatOption {
  label: string;
  suffix: string;
  format: ExportFormat;
}

export interface DistroExportMenuView {
  title: string;
  fileNameLabel: string;
  fileNamePlaceholder: string;
  formatLabel: string;
  directoryLabel: string;
  directoryPlaceholder: string;
  chooseDirectory: string;
  submit: string;
  exporting: string;
  defaultFileName: string;
  defaultDirectory: string;
  formats: DistroExportFormatOption[];
  errors: {
    fileNameRequired: string;
    fileNameInvalid: string;
    fileNameSuffixNotAllowed: string;
    directoryRequired: string;
    noTauriDirectoryPicker: string;
  };
}

export interface DistroRowView {
  name: string;
  logoSrc: string;
  panelId: string;
  state: DistroStateBadge;
  isDefault: boolean;
  isProtected: boolean;
  protectedMessage: string | null;
  flavorVersion: string | null;
  flavorVersionCompact: boolean;
  versionLabel: string;
  actionsDisabled: boolean;
  terminateRunning: boolean;
  unregisterBusy: boolean;
  deleteLabel: string;
  settingDefault: boolean;
  expanded: boolean;
  expandLabel: string;
  exportLabel: string;
  details: DistroDetailItem[];
  exportMenu: DistroExportMenuView;
}

export type DistroWorkspaceSectionState =
  | "loading"
  | "recovering"
  | "empty"
  | "ready";

export interface DistroWorkspaceSectionView {
  title: string;
  state: DistroWorkspaceSectionState;
  count: number;
  emptyTitle: string | null;
  emptyMessage: string | null;
  rows: DistroRowView[];
}

export interface DistroWorkspaceView {
  notices: DistroWorkspaceNotice[];
  refreshButton: {
    label: string;
    refreshingLabel: string;
    refreshing: boolean;
    disabled: boolean;
  };
  shutdownButton: {
    label: string;
    refreshingLabel: string;
    running: boolean;
    disabled: boolean;
  };
  section: DistroWorkspaceSectionView;
}

export interface DistroWorkspaceCallbacks {
  refresh: () => Promise<void>;
  shutdownAll: () => Promise<void>;
  terminate: (distroName: string) => Promise<void>;
  setDefault: (distroName: string) => Promise<void>;
  openTerminal: (distroName: string) => Promise<void>;
  openExplorer: (distroName: string) => Promise<void>;
  openVscode: (distroName: string) => Promise<void>;
  unregister: (distroName: string) => Promise<void>;
  chooseExportDirectory: (
    distroName: string,
    defaultPath?: string,
  ) => Promise<string | null>;
  submitExport: (
    distroName: string,
    file: string,
    format: ExportFormat,
    logoSrc: string,
  ) => Promise<boolean>;
  toggleExpanded: (distroName: string) => Promise<void>;
}

export const shutdownAllScope: Extract<
  ActionOverlayScope,
  { kind: "distros-list" }
> = {
  kind: "distros-list",
  operation: "shutdown-all",
};

export function createDistroRowScope(
  distroName: string,
  operation: Extract<ActionOverlayScope, { kind: "distro-row" }>["operation"],
): Extract<ActionOverlayScope, { kind: "distro-row" }> {
  return {
    kind: "distro-row",
    distroName,
    operation,
  };
}
