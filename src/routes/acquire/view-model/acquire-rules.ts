import { formatBytes } from "$lib/shared/format";
import { getWindowsParentPath, joinWindowsPath } from "$lib/shared/windows-path";
import type { AppCopy } from "$lib/i18n";
import type { FileSystemPathProbeState, PathVolumeSpaceProbeState } from "$lib/probes/path-input-probes";
import type { QueryEntry } from "$lib/query-cache";
import type { OnlineDistro } from "$lib/tauri/wsl";

export type AcquireMode = "store" | "import";
export type ImportKind = "archive" | "vhdx";
export type OnlineListState = "loading" | "recovering" | "ready" | "error" | "empty";
export type SpaceTone = "neutral" | "info" | "success" | "error";
export type InstallDraft = { name: string; location: string; vhdSize: string; fixedVhd: boolean };
export type ImportDraft = { name: string; file: string; location: string };
export type TargetProbe = { location: FileSystemPathProbeState; space: PathVolumeSpaceProbeState };

type DiskSize = { state: "empty" | "invalid" | "valid"; bytes: number | null };
type TargetOptions = { allowExistingDirectory?: boolean; emptyMessage?: string; probeFailedMessage?: string; existsMessage?: string };

const GB = 1024 ** 3;
const MIN_FREE = 5 * GB;
const MIN_VHD = 15 * GB;

export const acquireModeOrder: AcquireMode[] = ["store", "import"];
export const createInstallDraft = (name = "", location = ""): InstallDraft => ({ name, location, vhdSize: "", fixedVhd: false });
export const createImportDraft = (): ImportDraft => ({ name: "", file: "", location: "" });
export const createDistroTargetLocation = (root: string, name: string): string => joinWindowsPath(root, name) ?? "";
export const formatVhdSizeGb = (value: string): string | null => value.trim() ? `${value.trim()}GB` : null;
export const findOnlineDistro = (distros: OnlineDistro[], name: string | null) => name ? distros.find((distro) => distro.name === name) ?? null : null;
export const detectImportKind = (path: string): ImportKind | null => /\.(tar|tar\.gz|tar\.xz)$/i.test(path.trim()) ? "archive" : /\.vhdx$/i.test(path.trim()) ? "vhdx" : null;
export const deriveDistroNameFromImportFile = (path: string): string => (getWindowsFileName(path) ?? "").replace(/\.(tar\.gz|tar\.xz|tar|vhdx)$/i, "").trim();
export const getVhdxTargetDirectory = (draft: ImportDraft): string | null => isDirectVhdxImport(draft) ? null : finalLocation(draft.location);

export function getOnlineListState(entry: QueryEntry<OnlineDistro[]>): OnlineListState {
  const count = entry.data?.length ?? 0;
  if (entry.activity === "loading" && count === 0) return "loading";
  if (entry.isRecovering && count === 0) return "recovering";
  if (entry.hasError && count === 0) return "error";
  return count === 0 ? "empty" : "ready";
}

export function validateInstall(input: { copy: AppCopy; enabled: boolean; draft: InstallDraft; target: TargetProbe; nameDuplicate: boolean; nameProbePending: boolean; nameProbeError: string | null; hasTauriBridge: boolean }) {
  const { copy, draft, enabled, target } = input;
  const nameError = enabled ? validateName(copy, draft.name, input.nameDuplicate, input.nameProbeError) : null;
  const vhdSizeError = enabled ? validateVhdSize(copy, draft) : null;
  const locationError = enabled ? validateTargetDirectory(copy, draft.location, target) : null;
  return {
    nameError,
    vhdSizeError,
    locationError,
    disabled: !input.hasTauriBridge || !enabled || input.nameProbePending || !!nameError || !!vhdSizeError || !!locationError || isTargetChecking(draft.location, target),
  };
}

export function validateImport(input: { copy: AppCopy; draft: ImportDraft; kind: ImportKind | null; target: TargetProbe; fileProbe: FileSystemPathProbeState; nameDuplicate: boolean; nameProbePending: boolean; nameProbeError: string | null; hasTauriBridge: boolean }) {
  const { copy, draft, fileProbe, target } = input;
  const directVhdxImport = isDirectVhdxImport(draft);
  const nameError = validateName(copy, draft.name, input.nameDuplicate, input.nameProbeError);
  const fileError = validateImportFile(copy, draft.file, input.kind) ?? validateImportFileProbe(copy, fileProbe);
  const locationError = validateTargetDirectory(copy, draft.location, target, {
    allowExistingDirectory: directVhdxImport,
    emptyMessage: copy.acquire.validation.targetDirectoryRequired,
    probeFailedMessage: copy.acquire.validation.targetDirectoryProbeFailed,
    existsMessage: copy.acquire.validation.targetDirectoryExists,
  });
  return {
    nameError,
    fileError,
    locationError,
    finalLocation: finalLocation(draft.location),
    directVhdxImport,
    disabled: !input.hasTauriBridge || input.nameProbePending || !!nameError || !!fileError || !!locationError || (draft.file.trim().length > 0 && fileProbe.status === "loading") || isTargetChecking(draft.location, target),
  };
}

export function getSpaceNotice(input: { copy: AppCopy; location: string; target: TargetProbe; emptyMessage: string }): { tone: SpaceTone; message: string } {
  if (finalLocation(input.location) === null) return { tone: "neutral", message: input.emptyMessage };
  const { copy, target } = input;
  const { space } = target;
  if (space.status === "loading") return { tone: "info", message: copy.acquire.validation.diskSpaceChecking };
  if (space.status === "error") return { tone: "error", message: space.errorMessage ?? copy.acquire.validation.diskSpaceReadFailed };
  if (space.status !== "ready" || space.freeBytes === null) return { tone: "info", message: copy.acquire.validation.diskSpaceWaiting };
  const free = formatBytes(space.freeBytes, 2, copy.common.missing);
  return space.freeBytes <= MIN_FREE
    ? { tone: "error", message: copy.acquire.validation.diskSpaceNotEnough(free) }
    : { tone: "success", message: copy.acquire.validation.diskSpaceEnough(free) };
}

function validateName(copy: AppCopy, name: string, duplicate: boolean, probeError: string | null) {
  if (!name.trim()) return copy.acquire.validation.nameRequired;
  if (/\s/.test(name)) return copy.acquire.validation.nameNoWhitespace;
  if (probeError) return copy.acquire.validation.nameProbeFailed;
  return duplicate ? copy.acquire.validation.nameExists : null;
}

function validateVhdSize(copy: AppCopy, draft: InstallDraft): string | null {
  const parsed = parseDiskSize(draft.vhdSize);
  if (draft.fixedVhd && parsed.state === "empty") return copy.acquire.validation.fixedVhdSizeRequired;
  if (parsed.state === "invalid") return copy.acquire.validation.vhdSizeInteger;
  return parsed.bytes !== null && parsed.bytes < MIN_VHD ? copy.acquire.validation.vhdSizeMinimum : null;
}

function validateTargetDirectory(copy: AppCopy, value: string, target: TargetProbe, options: TargetOptions = {}): string | null {
  if (finalLocation(value) === null) return options.emptyMessage ?? copy.acquire.validation.installLocationRequired;
  if (target.location.status === "error") return target.location.errorMessage ?? options.probeFailedMessage ?? copy.acquire.validation.installLocationProbeFailed;
  if (target.location.status === "ready" && target.location.probe?.exists && !options.allowExistingDirectory) return options.existsMessage ?? copy.acquire.validation.installLocationExists;
  if (target.space.status === "error") return target.space.errorMessage ?? copy.acquire.validation.diskSpaceReadFailed;
  return target.space.status === "ready" && target.space.freeBytes !== null && target.space.freeBytes <= MIN_FREE
    ? copy.acquire.validation.diskSpaceInsufficient(formatBytes(MIN_FREE, 2, copy.common.missing))
    : null;
}

function validateImportFile(copy: AppCopy, file: string, kind: ImportKind | null) {
  if (!file.trim()) return copy.acquire.validation.importFileRequired;
  return kind === null ? copy.acquire.validation.importFileUnsupported : null;
}

function validateImportFileProbe(copy: AppCopy, probe: FileSystemPathProbeState) {
  if (probe.status === "idle" || probe.status === "loading") return null;
  if (probe.status === "error") return probe.errorMessage ?? copy.acquire.validation.importFileProbeFailed;
  if (!probe.probe?.exists) return copy.acquire.validation.importFileMissing;
  return probe.probe.isFile ? null : copy.acquire.validation.importFileNotFile;
}

function parseDiskSize(value: string): DiskSize {
  const trimmed = value.trim();
  if (!trimmed) return { state: "empty", bytes: null };
  if (!/^[1-9]\d*$/.test(trimmed)) return { state: "invalid", bytes: null };
  const numeric = Number(trimmed);
  return Number.isSafeInteger(numeric) ? { state: "valid", bytes: numeric * GB } : { state: "invalid", bytes: null };
}

function isTargetChecking(value: string, target: TargetProbe) {
  return finalLocation(value) !== null && (target.location.status === "loading" || (target.space.status !== "ready" && target.space.status !== "error"));
}

function isDirectVhdxImport(draft: ImportDraft) {
  const location = normalizeWindowsPath(draft.location);
  const sourceParent = normalizeWindowsPath(getWindowsParentPath(draft.file));
  return detectImportKind(draft.file) === "vhdx" && location.length > 0 && location === sourceParent;
}

function getWindowsFileName(path: string) {
  const trimmed = path.trim().replace(/[\\/]+$/, "");
  return trimmed ? trimmed.split(/[\\/]/).pop() ?? null : null;
}

function finalLocation(value: string) {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function normalizeWindowsPath(path: string | null) {
  return (path ?? "").trim().replace(/\//g, "\\").replace(/\\+$/, "").toLocaleLowerCase("en-US");
}
