import { isDistroRunning, normalizeDistroName } from "$lib/shared/distros";
import { listDistros } from "$lib/tauri/wsl";

export interface LiveDistroSnapshot {
  name: string;
  isRunning: boolean;
  isDefault: boolean;
}

export interface LiveDistro {
  find: (distroName: string) => LiveDistroSnapshot | null;
  exists: (distroName: string) => boolean;
  isStopped: (distroName: string) => boolean;
  hasAnyRunning: () => boolean;
  isDefault: (distroName: string) => boolean;
}

export async function loadLiveDistro(): Promise<LiveDistro> {
  const snapshots = (await listDistros()).map((distro) => ({
    name: distro.name,
    isRunning: isDistroRunning(distro),
    isDefault: distro.is_default,
  }));

  function find(distroName: string): LiveDistroSnapshot | null {
    const normalizedName = normalizeDistroName(distroName);

    if (normalizedName.length === 0) {
      return null;
    }

    return (
      snapshots.find(
        (snapshot) => normalizeDistroName(snapshot.name) === normalizedName,
      ) ?? null
    );
  }

  return {
    find,
    exists: (distroName) => find(distroName) !== null,
    isStopped: (distroName) => {
      const snapshot = find(distroName);
      return snapshot !== null && !snapshot.isRunning;
    },
    hasAnyRunning: () => snapshots.some((snapshot) => snapshot.isRunning),
    isDefault: (distroName) => find(distroName)?.isDefault === true,
  };
}
