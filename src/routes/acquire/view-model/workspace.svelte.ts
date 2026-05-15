import { open } from "@tauri-apps/plugin-dialog";
import { untrack } from "svelte";
import { get } from "svelte/store";
import { pushToast } from "$lib/feedback/toasts";
import { getCopy, i18nState } from "$lib/i18n";
import { longTaskState, submitAcquireTask } from "$lib/long-tasks";
import { queryCache, refreshAcquireWorkspace, type QueryCacheState } from "$lib/query-cache";
import { isDistroNameUsed } from "$lib/shared/distro-targets";
import { hasTauriBridge } from "$lib/shared/runtime";
import { getWindowsParentPath } from "$lib/shared/windows-path";
import { DEFAULT_INSTALL_LOCATION, getDefaultInstallLocation } from "$lib/tauri/settings";
import type { InstallOptions } from "$lib/tauri/wsl";
import { createAcquireProbeController, createIdleAcquireProbeState } from "../service/probes";
import {
  createDistroTargetLocation,
  createImportDraft,
  createInstallDraft,
  deriveDistroNameFromImportFile,
  detectImportKind,
  findOnlineDistro,
  formatVhdSizeGb,
  getOnlineListState,
  getSpaceNotice,
  getVhdxTargetDirectory,
  validateImport,
  validateInstall,
  type ImportDraft,
  type InstallDraft,
} from "./acquire-rules";

const selectedPath = (value: unknown) => (typeof value === "string" ? value : null);
const makeTarget = (root: string, name: string) => createDistroTargetLocation(root, name);
const pickDirectory = async (title: string, defaultPath?: string) => selectedPath(await open({ title, directory: true, multiple: false, defaultPath }));
const pickImportFile = async () => selectedPath(await open({ title: getCopy().acquire.dialogs.importFileTitle, directory: false, multiple: false, filters: [{ name: getCopy().acquire.dialogs.importFileFilterName, extensions: ["tar", "tar.gz", "tar.xz", "vhdx"] }] }));

export function createAcquireWorkspaceViewModel() {
  const probes = createAcquireProbeController();
  let queryState = $state<QueryCacheState>(get(queryCache));
  let defaultLocation = $state(DEFAULT_INSTALL_LOCATION);
  let selectedDistroName = $state<string | null>(null);
  let draft = $state<InstallDraft>(createInstallDraft());
  let importDraft = $state<ImportDraft>(createImportDraft());
  let refreshing = $state(false);
  let installSubmitting = $state(false);
  let importSubmitting = $state(false);
  let hasActiveLongTask = $state(get(longTaskState).hasActiveLongTask);
  let probeState = $state(createIdleAcquireProbeState());
  let copy = $state(getCopy());
  let installLocationTouched = false;
  let importNameTouched = false;
  let importLocationTouched = false;
  let disposed = false;

  const onlineDistros = $derived(queryState.onlineDistros.data ?? []);
  const installedDistros = $derived(queryState.distros.data ?? []);
  const selectedDistro = $derived(findOnlineDistro(onlineDistros, selectedDistroName));
  const detectedImportKind = $derived(detectImportKind(importDraft.file));
  const importNoun = $derived(detectedImportKind === "archive" ? copy.acquire.importNouns.archive : detectedImportKind === "vhdx" ? copy.acquire.importNouns.vhdx : copy.acquire.importNouns.generic);
  const nameProbePending = $derived(queryState.distros.data === null && queryState.distros.activity === "loading");
  const nameProbeError = $derived(queryState.distros.data === null && !nameProbePending && queryState.distros.hasError ? copy.acquire.validation.installedDistroListReadFailed : null);
  const validation = $derived(validateInstall({
    copy, enabled: selectedDistro !== null, draft, target: probeState.install,
    nameDuplicate: isDistroNameUsed(installedDistros, draft.name), nameProbePending, nameProbeError, hasTauriBridge: hasTauriBridge(),
  }));
  const importValidation = $derived(validateImport({
    copy, draft: importDraft, kind: detectedImportKind, target: probeState.importTarget, fileProbe: probeState.importFile,
    nameDuplicate: isDistroNameUsed(installedDistros, importDraft.name), nameProbePending, nameProbeError, hasTauriBridge: hasTauriBridge(),
  }));
  const spaceNotice = $derived(selectedDistro === null ? null : getSpaceNotice({ copy, location: draft.location, target: probeState.install, emptyMessage: copy.acquire.validation.installSpaceEmpty }));
  const importSpaceNotice = $derived(getSpaceNotice({ copy, location: importDraft.location, target: probeState.importTarget, emptyMessage: copy.acquire.validation.importSpaceEmpty }));
  const installSubmitDisabled = $derived(validation.disabled || installSubmitting || hasActiveLongTask);
  const importSubmitDisabled = $derived(importValidation.disabled || importSubmitting || hasActiveLongTask);

  const checkInstallLocation = (value: string) => selectedDistroName === null || !value.trim() ? probes.resetInstallTarget() : void probes.checkInstallTarget(value);
  const checkImportLocation = (value: string) => !value.trim() ? probes.resetImportTarget() : void probes.checkImportTarget(value);
  const setInstallLocation = (value: string, touched: boolean) => { installLocationTouched = touched; draft = { ...draft, location: value }; checkInstallLocation(value); };
  const setImportLocation = (value: string, touched: boolean) => { importLocationTouched = touched; importDraft = { ...importDraft, location: value }; checkImportLocation(value); };

  async function loadSettings() {
    const next = await getDefaultInstallLocation();
    if (disposed) return;
    defaultLocation = next;
    if (selectedDistroName !== null && !installLocationTouched) setInstallLocation(makeTarget(next, draft.name), false);
    if (detectedImportKind === "archive" && !importLocationTouched) setImportLocation(makeTarget(next, importDraft.name), false);
  }

  async function refreshOnlineDistros() {
    refreshing = true;
    try {
      const result = (await refreshAcquireWorkspace("manual")).onlineDistros;
      const ok = result?.kind === "fresh";
      pushToast({ tone: ok ? "success" : "error", title: ok ? copy.acquire.refresh.completedTitle : copy.acquire.refresh.failedTitle, message: ok ? copy.acquire.refresh.successMessage : copy.acquire.refresh.failedMessage });
    } finally {
      if (!disposed) refreshing = false;
    }
  }

  function selectDistro(name: string) {
    const distro = findOnlineDistro(onlineDistros, name);
    if (distro === null) return;
    selectedDistroName = distro.name;
    installLocationTouched = false;
    draft = createInstallDraft(distro.name, makeTarget(defaultLocation, distro.name));
    probes.resetInstallTarget();
    checkInstallLocation(draft.location);
  }

  function updateInstallName(name: string) {
    draft = { ...draft, name };
    if (!installLocationTouched) setInstallLocation(makeTarget(defaultLocation, name), false);
  }

  async function chooseInstallLocation() {
    if (!hasTauriBridge()) return;
    const root = await pickDirectory(copy.acquire.dialogs.installLocationTitle, draft.location || defaultLocation);
    if (root !== null) setInstallLocation(makeTarget(root, draft.name), true);
  }

  function updateImportName(name: string) {
    importNameTouched = true;
    importDraft = { ...importDraft, name };
    if (!importLocationTouched && detectedImportKind === "archive") setImportLocation(makeTarget(defaultLocation, name), false);
  }

  function updateImportFile(file: string) {
    const kind = detectImportKind(file);
    const name = importNameTouched ? importDraft.name : deriveDistroNameFromImportFile(file);
    const location = importLocationTouched ? importDraft.location : kind === "vhdx" ? getWindowsParentPath(file) ?? "" : kind === "archive" ? makeTarget(defaultLocation, name) : importDraft.location;
    importDraft = { ...importDraft, file, name, location };
    file.trim() ? void probes.checkImportFile(file) : probes.resetImportFile();
    checkImportLocation(location);
  }

  async function chooseImportFile() {
    if (!hasTauriBridge()) return;
    const file = await pickImportFile();
    if (file !== null) updateImportFile(file);
  }

  async function chooseImportLocation() {
    if (!hasTauriBridge()) return;
    const root = await pickDirectory(copy.acquire.dialogs.importRootTitle, importDraft.location || defaultLocation);
    if (root !== null) setImportLocation(makeTarget(root, importDraft.name), true);
  }

  function clearInstall() {
    selectedDistroName = null;
    installLocationTouched = false;
    draft = createInstallDraft();
    probes.resetInstallTarget();
  }

  function clearImport() {
    importNameTouched = false;
    importLocationTouched = false;
    importDraft = createImportDraft();
    probes.resetImportTarget();
    probes.resetImportFile();
  }

  async function startInstall() {
    if (selectedDistro === null || installSubmitDisabled) return pushToast({ tone: "error", title: copy.acquire.toasts.cannotStartInstallTitle, message: copy.acquire.toasts.installValidationFallback });
    const displayName = draft.name.trim();
    const options: InstallOptions = { name: displayName, location: draft.location.trim(), vhdSize: formatVhdSizeGb(draft.vhdSize), fixedVhd: draft.fixedVhd };
    installSubmitting = true;
    try {
      if (await submitAcquireTask({ operation: "install", distro: selectedDistro.name, displayName, location: draft.location.trim(), options })) clearInstall();
    } finally {
      if (!disposed) installSubmitting = false;
    }
  }

  async function startImport() {
    if (importValidation.disabled || detectedImportKind === null || importSubmitDisabled) return pushToast({ tone: "error", title: copy.acquire.toasts.cannotStartImportTitle, message: copy.acquire.toasts.importValidationFallback });
    importSubmitting = true;
    try {
      const completed = detectedImportKind === "archive"
        ? await submitAcquireTask({ operation: "importArchive", displayName: importDraft.name.trim(), location: importDraft.location.trim(), file: importDraft.file.trim() })
        : await submitAcquireTask({ operation: "importVhd", displayName: importDraft.name.trim(), sourceVhdx: importDraft.file.trim(), targetDirectory: getVhdxTargetDirectory(importDraft) });
      if (completed) clearImport();
    } finally {
      if (!disposed) importSubmitting = false;
    }
  }

  $effect(() => untrack(() => {
    const unQuery = queryCache.subscribe((state) => {
      queryState = state;
      if (selectedDistroName !== null && state.onlineDistros.data && findOnlineDistro(state.onlineDistros.data, selectedDistroName) === null) clearInstall();
    });
    const unLongTask = longTaskState.subscribe((state) => (hasActiveLongTask = state.hasActiveLongTask));
    const unI18n = i18nState.subscribe((state) => (copy = state.copy));
    const unProbes = probes.subscribe((state) => (probeState = state));
    void loadSettings();
    void refreshAcquireWorkspace("page-enter");
    return () => { disposed = true; unQuery(); unLongTask(); unI18n(); unProbes(); probes.dispose(); };
  }));

  return {
    get queryState() { return queryState; }, get onlineDistros() { return onlineDistros; }, get onlineState() { return getOnlineListState(queryState.onlineDistros); },
    get selectedDistro() { return selectedDistro; }, get selectedDistroName() { return selectedDistroName; }, get draft() { return draft; },
    get importDraft() { return importDraft; }, get detectedImportKind() { return detectedImportKind; }, get importNoun() { return importNoun; },
    get refreshing() { return refreshing; }, get refreshDisabled() { return refreshing || queryState.onlineDistros.activity === "loading"; },
    get validation() { return validation; }, get importValidation() { return importValidation; }, get spaceNotice() { return spaceNotice; }, get importSpaceNotice() { return importSpaceNotice; },
    get installSubmitting() { return installSubmitting; }, get importSubmitting() { return importSubmitting; }, get installSubmitDisabled() { return installSubmitDisabled; }, get importSubmitDisabled() { return importSubmitDisabled; },
    callbacks: { refreshOnlineDistros, selectDistro, updateName: updateInstallName, updateLocation: (value: string) => setInstallLocation(value, true), chooseLocation: chooseInstallLocation, updateVhdSize: (vhdSize: string) => (draft = { ...draft, vhdSize }), setFixedVhd: (fixedVhd: boolean) => (draft = { ...draft, fixedVhd }), startInstall, updateImportName, updateImportFile, updateImportRoot: (value: string) => setImportLocation(value, true), chooseImportFile, chooseImportRoot: chooseImportLocation, startImport },
  };
}

export type AcquireWorkspaceViewModel = ReturnType<typeof createAcquireWorkspaceViewModel>;
