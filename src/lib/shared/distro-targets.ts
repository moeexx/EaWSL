import type { FileSystemPathProbe } from "$lib/tauri/system";
import type { DistroInfo } from "$lib/tauri/wsl";

import { findDistroByName, normalizeDistroName } from "./distros";
import { getExplicitErrorMessage } from "./runtime";

type ProbeFileSystemPath = (
  path: string,
  childLimit?: number | null,
) => Promise<FileSystemPathProbe>;

export type DistroTargetDirectoryAvailability =
  | {
      kind: "available";
    }
  | {
      kind: "exists";
    }
  | {
      kind: "failed";
      message: string | null;
    };

export function isDistroNameUsed(
  distros: DistroInfo[],
  distroName: string,
): boolean {
  return normalizeDistroName(distroName).length > 0
    ? findDistroByName(distros, distroName) !== null
    : false;
}

export async function checkDistroTargetDirectoryAvailable(input: {
  location: string;
  probeFileSystemPath: ProbeFileSystemPath;
}): Promise<DistroTargetDirectoryAvailability> {
  try {
    const probe = await input.probeFileSystemPath(input.location, null);
    return probe.exists ? { kind: "exists" } : { kind: "available" };
  } catch (error) {
    return {
      kind: "failed",
      message: getExplicitErrorMessage(error),
    };
  }
}
