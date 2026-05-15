import { get, writable } from "svelte/store";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { hasTauriBridge } from "$lib/shared/runtime";

type ResizeDirection =
  | "East"
  | "North"
  | "NorthEast"
  | "NorthWest"
  | "South"
  | "SouthEast"
  | "SouthWest"
  | "West";

const SIDEBAR_COLLAPSED_KEY = "eawsl.shell.sidebar-collapsed";

export interface ShellUiState {
  sidebarCollapsed: boolean;
  taskTrayExpanded: boolean;
  windowMaximized: boolean;
}

const initialState: ShellUiState = {
  sidebarCollapsed: false,
  taskTrayExpanded: false,
  windowMaximized: false,
};

const store = writable<ShellUiState>(initialState);

let started = false;
let disposed = false;
let unlistenPromises: Array<Promise<() => void>> = [];

function updateWindowMeta(patch: Pick<ShellUiState, "windowMaximized">): void {
  store.update((state) => ({
    ...state,
    ...patch,
  }));
}

function persistSidebarCollapsed(value: boolean): void {
  if (typeof localStorage === "undefined") {
    return;
  }

  localStorage.setItem(SIDEBAR_COLLAPSED_KEY, value ? "1" : "0");
}

async function syncWindowState(): Promise<void> {
  if (!hasTauriBridge()) {
    updateWindowMeta({
      windowMaximized: false,
    });
    return;
  }

  const appWindow = getCurrentWindow();
  const windowMaximized = await appWindow.isMaximized().catch(() => false);

  updateWindowMeta({
    windowMaximized,
  });
}

export const shellUiState = {
  subscribe: store.subscribe,
};

export function startShellUi(): void {
  if (started) {
    return;
  }

  started = true;
  disposed = false;

  if (typeof localStorage !== "undefined") {
    const stored = localStorage.getItem(SIDEBAR_COLLAPSED_KEY);
    if (stored !== null) {
      store.update((state) => ({
        ...state,
        sidebarCollapsed: stored === "1",
      }));
    }
  }

  if (!hasTauriBridge()) {
    return;
  }

  const appWindow = getCurrentWindow();
  void syncWindowState();

  unlistenPromises = [
    appWindow.onResized(() => {
      if (!disposed) {
        void syncWindowState();
      }
    }),
  ];
}

export function stopShellUi(): void {
  if (!started) {
    return;
  }

  disposed = true;
  started = false;

  for (const promise of unlistenPromises) {
    void promise.then((unlisten) => {
      unlisten();
    });
  }

  unlistenPromises = [];
}

function setSidebarCollapsed(value: boolean): void {
  persistSidebarCollapsed(value);
  store.update((state) => ({
    ...state,
    sidebarCollapsed: value,
  }));
}

export function toggleSidebarCollapsed(): void {
  setSidebarCollapsed(!get(store).sidebarCollapsed);
}

function setTaskTrayExpanded(value: boolean): void {
  store.update((state) => ({
    ...state,
    taskTrayExpanded: value,
  }));
}

export function toggleTaskTrayExpanded(): void {
  setTaskTrayExpanded(!get(store).taskTrayExpanded);
}

export async function minimizeWindow(): Promise<void> {
  if (!hasTauriBridge()) {
    return;
  }

  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow(): Promise<void> {
  if (!hasTauriBridge()) {
    return;
  }

  const appWindow = getCurrentWindow();
  await appWindow.toggleMaximize();
  await syncWindowState();
}

export async function closeWindow(): Promise<void> {
  if (!hasTauriBridge()) {
    return;
  }

  await getCurrentWindow().close();
}

export async function startResizeDrag(direction: ResizeDirection): Promise<void> {
  if (!hasTauriBridge() || get(store).windowMaximized) {
    return;
  }

  await getCurrentWindow().startResizeDragging(direction);
}
