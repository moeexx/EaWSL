import { writable } from "svelte/store";

import {
  getPathVolumeSpace,
  probeFileSystemPath,
  type FileSystemPathProbe,
  type PathVolumeSpace,
} from "$lib/tauri/system";
import { getExplicitErrorMessage, hasTauriBridge } from "$lib/shared/runtime";

const PATH_VOLUME_SPACE_TTL_MS = 5 * 60 * 1000;

export type PathVolumeSpaceStatus = "idle" | "loading" | "ready" | "error";
export type FileSystemPathProbeStatus = "idle" | "loading" | "ready" | "error";

export interface PathVolumeSpaceProbeState {
  status: PathVolumeSpaceStatus;
  volumeRoot: string | null;
  freeBytes: number | null;
  errorMessage: string | null;
}

export interface FileSystemPathProbeState {
  status: FileSystemPathProbeStatus;
  probe: FileSystemPathProbe | null;
  errorMessage: string | null;
}

interface CachedPathVolumeSpace extends PathVolumeSpace {
  expiresAt: number;
}

const pathVolumeSpaceCache = new Map<string, CachedPathVolumeSpace>();
const pathVolumeSpaceRequests = new Map<string, Promise<PathVolumeSpace>>();

async function checkPathVolumeSpace(path: string): Promise<PathVolumeSpace> {
  const trimmed = path.trim();

  pruneExpiredPathVolumeSpaceCache();

  const requestKey = getPathVolumeSpaceRequestKey(trimmed);
  const cached = getFreshPathVolumeSpaceForPath(trimmed);

  if (cached) {
    return cached;
  }

  const inFlight = pathVolumeSpaceRequests.get(requestKey);

  if (inFlight) {
    return inFlight;
  }

  const request = getPathVolumeSpace(trimmed)
    .then((result) => {
      writePathVolumeSpaceCache(result, requestKey);
      return result;
    })
    .finally(() => {
      if (pathVolumeSpaceRequests.get(requestKey) === request) {
        pathVolumeSpaceRequests.delete(requestKey);
      }
    });

  pathVolumeSpaceRequests.set(requestKey, request);

  return request;
}

export function createPathVolumeSpaceProbe() {
  const store = writable<PathVolumeSpaceProbeState>(
    createIdlePathVolumeSpaceState(),
  );
  let requestToken = 0;

  async function check(path: string): Promise<void> {
    const trimmed = path.trim();
    const token = ++requestToken;

    if (trimmed.length === 0) {
      store.set(createIdlePathVolumeSpaceState());
      return;
    }

    if (!hasTauriBridge()) {
      store.set({
        status: "error",
        volumeRoot: null,
        freeBytes: null,
        errorMessage: null,
      });
      return;
    }

    const cached = getFreshPathVolumeSpaceForPath(trimmed);

    if (cached) {
      store.set({
        status: "ready",
        volumeRoot: cached.volumeRoot,
        freeBytes: cached.freeBytes,
        errorMessage: null,
      });
      return;
    }

    store.set({
      status: "loading",
      volumeRoot: null,
      freeBytes: null,
      errorMessage: null,
    });

    try {
      const volumeSpace = await checkPathVolumeSpace(trimmed);

      if (token !== requestToken) {
        return;
      }

      store.set({
        status: "ready",
        volumeRoot: volumeSpace.volumeRoot,
        freeBytes: volumeSpace.freeBytes,
        errorMessage: null,
      });
    } catch (error) {
      if (token !== requestToken) {
        return;
      }

      store.set({
        status: "error",
        volumeRoot: null,
        freeBytes: null,
        errorMessage: getExplicitErrorMessage(error),
      });
    }
  }

  function reset(): void {
    requestToken += 1;
    store.set(createIdlePathVolumeSpaceState());
  }

  function cancel(): void {
    requestToken += 1;
  }

  return {
    subscribe: store.subscribe,
    check,
    reset,
    cancel,
  };
}

export function createFileSystemPathProbe() {
  return createPathInputProbe<FileSystemPathProbeState>({
    idle: createIdleFileSystemPathProbeState,
    loading: () => ({
      status: "loading",
      probe: null,
      errorMessage: null,
    }),
    bridgeError: () => ({
      status: "error",
      probe: null,
      errorMessage: null,
    }),
    read: async (path, childLimit) => ({
      status: "ready",
      probe: await probeFileSystemPath(path, childLimit),
      errorMessage: null,
    }),
    error: (error) => ({
      status: "error",
      probe: null,
      errorMessage: getExplicitErrorMessage(error),
    }),
  });
}

function createPathInputProbe<TState>(input: {
  idle: () => TState;
  loading: () => TState;
  bridgeError: () => TState;
  read: (path: string, childLimit: number | null) => Promise<TState>;
  error: (error: unknown) => TState;
}) {
  const store = writable<TState>(input.idle());
  let requestToken = 0;

  async function check(
    path: string | null,
    childLimit: number | null = null,
  ): Promise<void> {
    const token = ++requestToken;
    const trimmed = path?.trim() ?? "";

    if (trimmed.length === 0) {
      store.set(input.idle());
      return;
    }

    if (!hasTauriBridge()) {
      store.set(input.bridgeError());
      return;
    }

    store.set(input.loading());

    try {
      const state = await input.read(trimmed, childLimit);
      if (token !== requestToken) return;
      store.set(state);
    } catch (error) {
      if (token !== requestToken) return;
      store.set(input.error(error));
    }
  }

  function reset(): void {
    requestToken += 1;
    store.set(input.idle());
  }

  function cancel(): void {
    requestToken += 1;
  }

  return {
    subscribe: store.subscribe,
    check,
    reset,
    cancel,
  };
}

export function createIdleFileSystemPathProbeState(): FileSystemPathProbeState {
  return {
    status: "idle",
    probe: null,
    errorMessage: null,
  };
}

export function createIdlePathVolumeSpaceState(): PathVolumeSpaceProbeState {
  return {
    status: "idle",
    volumeRoot: null,
    freeBytes: null,
    errorMessage: null,
  };
}

function getFreshPathVolumeSpace(
  key: string,
  now = Date.now(),
): CachedPathVolumeSpace | null {
  const cached = pathVolumeSpaceCache.get(key);

  if (!cached || cached.expiresAt <= now) {
    return null;
  }

  return cached;
}

function pruneExpiredPathVolumeSpaceCache(now = Date.now()): void {
  for (const [key, cached] of pathVolumeSpaceCache) {
    if (cached.expiresAt <= now) {
      pathVolumeSpaceCache.delete(key);
    }
  }
}

function getFreshPathVolumeSpaceForPath(path: string): PathVolumeSpace | null {
  const cached = getFreshPathVolumeSpace(getPathVolumeSpaceRequestKey(path));

  if (!cached) {
    return null;
  }

  return {
    volumeRoot: cached.volumeRoot,
    freeBytes: cached.freeBytes,
  };
}

function getPathVolumeSpaceRequestKey(path: string): string {
  return resolveLikelyVolumeKey(path) ?? normalizePathKey(path);
}

function writePathVolumeSpaceCache(
  result: PathVolumeSpace,
  requestKey: string,
): void {
  const cached = {
    ...result,
    expiresAt: Date.now() + PATH_VOLUME_SPACE_TTL_MS,
  };
  const volumeKey = normalizeVolumeKey(result.volumeRoot);

  pathVolumeSpaceCache.set(volumeKey, cached);

  if (requestKey === volumeKey || requestKey.startsWith("path:")) {
    pathVolumeSpaceCache.set(requestKey, cached);
  }
}

function resolveLikelyVolumeKey(path: string): string | null {
  const driveMatch = path.match(/^([a-zA-Z]):(?:[\\/]|$)/);

  if (driveMatch) {
    return normalizeVolumeKey(`${driveMatch[1]}:\\`);
  }

  const uncMatch = path.match(/^[\\/]{2}([^\\/]+)[\\/]([^\\/]+)(?:[\\/]|$)/);

  if (uncMatch) {
    return normalizeVolumeKey(`\\\\${uncMatch[1]}\\${uncMatch[2]}\\`);
  }

  return null;
}

function normalizePathKey(path: string): string {
  return `path:${path.replace(/\//g, "\\").trim().toLowerCase()}`;
}

function normalizeVolumeKey(volumeRoot: string): string {
  return volumeRoot
    .replace(/\//g, "\\")
    .replace(/[\\/]+$/, "\\")
    .toLowerCase();
}
