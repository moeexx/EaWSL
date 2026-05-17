import type { DistroInfo } from "$lib/tauri/wsl";

const uiActionHiddenDistroNames = new Set(["docker-desktop"]);

export function findDistroByName(
  distros: DistroInfo[],
  distroName: string,
): DistroInfo | null {
  const normalizedTarget = normalizeDistroName(distroName);

  if (normalizedTarget.length === 0) {
    return null;
  }

  return (
    distros.find(
      (distro) => normalizeDistroName(distro.name) === normalizedTarget,
    ) ?? null
  );
}

export function findDefaultDistro(distros: DistroInfo[]): DistroInfo | null {
  return distros.find(isDefaultDistro) ?? null;
}

export function isDistroRunning(distro: DistroInfo): boolean {
  return distro.state === "Running";
}

function isDefaultDistro(distro: DistroInfo): boolean {
  return distro.is_default;
}

export function hasHiddenRowActions(distro: DistroInfo | string): boolean {
  const name = typeof distro === "string" ? distro : distro.name;
  return uiActionHiddenDistroNames.has(normalizeDistroName(name));
}

export function sortUiActionHiddenDistrosLast(
  distros: DistroInfo[],
): DistroInfo[] {
  const regularDistros: DistroInfo[] = [];
  const actionHiddenDistros: DistroInfo[] = [];

  for (const distro of distros) {
    if (hasHiddenRowActions(distro)) {
      actionHiddenDistros.push(distro);
    } else {
      regularDistros.push(distro);
    }
  }

  return [...regularDistros, ...actionHiddenDistros];
}

export function normalizeDistroName(name: string): string {
  return name.trim().toLowerCase();
}
