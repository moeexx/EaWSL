import { writable } from "svelte/store";
import {
  createFileSystemPathProbe,
  createIdleFileSystemPathProbeState,
  createIdlePathVolumeSpaceState,
  createPathVolumeSpaceProbe,
  type FileSystemPathProbeState,
  type PathVolumeSpaceProbeState,
} from "$lib/probes/path-input-probes";

export type TargetProbeState = { location: FileSystemPathProbeState; space: PathVolumeSpaceProbeState };
export type AcquireProbeState = { install: TargetProbeState; importTarget: TargetProbeState; importFile: FileSystemPathProbeState };

const idleTarget = (): TargetProbeState => ({ location: createIdleFileSystemPathProbeState(), space: createIdlePathVolumeSpaceState() });
export const createIdleAcquireProbeState = (): AcquireProbeState => ({ install: idleTarget(), importTarget: idleTarget(), importFile: createIdleFileSystemPathProbeState() });

function createTargetProbe() {
  const location = createFileSystemPathProbe();
  const space = createPathVolumeSpaceProbe();
  let lastPath: string | null = null;
  async function check(path: string | null) {
    const trimmed = path?.trim() ?? "";
    if (!trimmed) return reset();
    if (trimmed === lastPath) return;
    lastPath = trimmed;
    void location.check(trimmed);
    await space.check(trimmed);
  }
  function reset() {
    lastPath = null;
    location.reset();
    space.reset();
  }
  return { location, space, check, reset, cancel: () => { location.cancel(); space.cancel(); } };
}

export function createAcquireProbeController() {
  let current = createIdleAcquireProbeState();
  const store = writable<AcquireProbeState>(current);
  const install = createTargetProbe();
  const importTarget = createTargetProbe();
  const importFile = createFileSystemPathProbe();
  const patch = (next: Partial<AcquireProbeState>) => { current = { ...current, ...next }; store.set(current); };
  const unsubs = [
    install.location.subscribe((location) => patch({ install: { ...current.install, location } })),
    install.space.subscribe((space) => patch({ install: { ...current.install, space } })),
    importTarget.location.subscribe((location) => patch({ importTarget: { ...current.importTarget, location } })),
    importTarget.space.subscribe((space) => patch({ importTarget: { ...current.importTarget, space } })),
    importFile.subscribe((state) => patch({ importFile: state })),
  ];
  return {
    subscribe: store.subscribe,
    checkInstallTarget: install.check,
    checkImportTarget: importTarget.check,
    resetInstallTarget: install.reset,
    resetImportTarget: importTarget.reset,
    checkImportFile: importFile.check,
    resetImportFile: importFile.reset,
    dispose() { unsubs.forEach((unsub) => unsub()); install.cancel(); importTarget.cancel(); importFile.cancel(); },
  };
}
