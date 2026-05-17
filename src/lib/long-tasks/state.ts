import { writable } from "svelte/store";

import {
  listenTransferProgress,
  type DistroProgressEvent,
  type ProgressPhase,
  type ProgressState,
  type ProgressValue,
} from "$lib/tauri/wsl";
import { hasTauriBridge } from "$lib/shared/runtime";
import {
  getLongTasks,
  saveLongTasks,
  type PersistedLongTask,
} from "$lib/tauri/long-tasks";

export type LongTaskStatus = "started" | "running" | "completed" | "failed";
export type LongTaskOperation =
  | "install"
  | "importArchive"
  | "importVhd"
  | "export";

export interface LongTask {
  requestId: string;
  distro: string;
  operation: LongTaskOperation;
  status: LongTaskStatus;
  phase: ProgressPhase | null;
  percent: number | null;
  startedAt: Date;
  endedAt: Date | null;
  error: string | null;
  location: string | null;
  logoSrc: string;
  interrupted: boolean;
}

export interface StartTaskInput {
  requestId: string;
  distro: string;
  operation: LongTaskOperation;
  location: string | null;
  logoSrc: string;
}

interface LongTaskState {
  tasks: LongTask[];
  hasActiveLongTask: boolean;
}

const store = writable<LongTaskState>({
  tasks: [],
  hasActiveLongTask: false,
});

let listening = false;
let disposed = false;
let unlistenPromises: Array<Promise<() => void>> = [];
let hydratePromise: Promise<void> | null = null;
let persistQueue: Promise<void> = Promise.resolve();

export const longTaskState = {
  subscribe: store.subscribe,
};

function syncActiveFlag(tasks: LongTask[]): LongTaskState {
  return {
    tasks,
    hasActiveLongTask: tasks.some(
      (task) => task.status === "started" || task.status === "running",
    ),
  };
}

function updateTasks(updater: (tasks: LongTask[]) => LongTask[]): LongTask[] {
  let nextTasks: LongTask[] = [];
  store.update((state) => {
    nextTasks = updater(state.tasks);
    return syncActiveFlag(nextTasks);
  });
  return nextTasks;
}

async function updateTaskAndSave(
  requestId: string,
  updater: (task: LongTask) => LongTask,
): Promise<void> {
  await ensureLongTasksHydrated();
  const tasks = updateTasks((currentTasks) =>
    currentTasks.map((task) =>
      task.requestId === requestId ? updater(task) : task,
    ),
  );
  await persistLongTasks(tasks);
}

function updateTaskBestEffort(
  requestId: string,
  updater: (task: LongTask) => LongTask,
): void {
  const tasks = updateTasks((currentTasks) =>
    currentTasks.map((task) =>
      task.requestId === requestId ? updater(task) : task,
    ),
  );
  persistLongTasksBestEffort(tasks);
}

function extractProgressState(value: ProgressValue): ProgressState | null {
  return "Status" in value ? value.Status : null;
}

function extractProgressPercent(value: ProgressValue): number | null {
  return "Percent" in value ? value.Percent : null;
}

export async function startTask(input: StartTaskInput): Promise<void> {
  await ensureLongTasksHydrated();
  const tasks = updateTasks((currentTasks) => [
    {
      requestId: input.requestId,
      distro: input.distro,
      operation: input.operation,
      status: "started",
      phase: null,
      percent: null,
      startedAt: new Date(),
      endedAt: null,
      error: null,
      location: input.location,
      logoSrc: input.logoSrc,
      interrupted: false,
    },
    ...currentTasks,
  ]);
  await persistLongTasks(tasks);
}

export async function failTask(
  requestId: string,
  error: string,
): Promise<void> {
  await updateTaskAndSave(requestId, (task) => ({
    ...task,
    status: "failed",
    endedAt: task.endedAt ?? new Date(),
    error,
    interrupted: false,
  }));
}

export async function completeTask(requestId: string): Promise<void> {
  await updateTaskAndSave(requestId, (task) => ({
    ...task,
    status: "completed",
    endedAt: task.endedAt ?? new Date(),
    error: null,
    interrupted: false,
  }));
}

export function applyLongTaskProgress(payload: DistroProgressEvent): void {
  const progressState = extractProgressState(payload.progress.value);
  const progressPercent = extractProgressPercent(payload.progress.value);

  updateTaskBestEffort(payload.requestId, (task) => {
    const phaseChanged = task.phase !== payload.progress.phase;
    const nextTask: LongTask = {
      ...task,
      phase: payload.progress.phase,
      interrupted: false,
    };

    if (progressPercent !== null) {
      nextTask.status = "running";
      nextTask.percent = progressPercent;
    }

    if (progressState === "Started") {
      nextTask.status = "started";
      if (phaseChanged) {
        nextTask.percent = null;
      }
    } else if (progressState === "Running") {
      nextTask.status = "running";
    } else if (progressState === "Completed") {
      nextTask.status = "completed";
      nextTask.endedAt = nextTask.endedAt ?? new Date();
    }

    return nextTask;
  });
}

export function startLongTaskFeed(): void {
  if (listening || !hasTauriBridge()) {
    return;
  }

  void ensureLongTasksHydrated().catch(() => undefined);
  listening = true;
  disposed = false;
  unlistenPromises = [
    listenTransferProgress((payload) => {
      if (!disposed) {
        applyLongTaskProgress(payload);
      }
    }),
  ];
}

export function stopLongTaskFeed(): void {
  if (!listening) {
    return;
  }

  disposed = true;
  listening = false;

  for (const promise of unlistenPromises) {
    void promise.then((unlisten) => {
      unlisten();
    });
  }

  unlistenPromises = [];
}

async function ensureLongTasksHydrated(): Promise<void> {
  if (!hasTauriBridge()) {
    return;
  }

  hydratePromise ??= hydrateLongTasks().catch((error) => {
    hydratePromise = null;
    throw error;
  });
  return hydratePromise;
}

async function hydrateLongTasks(): Promise<void> {
  const persistedTasks = await getLongTasks();
  const hadActiveTasks = persistedTasks.some(
    (task) => task.status === "started" || task.status === "running",
  );
  const tasks = persistedTasks.map(mapPersistedTask);

  store.set(syncActiveFlag(tasks));

  if (hadActiveTasks) {
    await persistLongTasks(tasks);
  }
}

function mapPersistedTask(task: PersistedLongTask): LongTask {
  const startedAt = parsePersistedDate(task.startedAt);
  const endedAt =
    task.endedAt === null ? null : parsePersistedDate(task.endedAt);
  const active = task.status === "started" || task.status === "running";

  return {
    requestId: task.requestId,
    distro: task.distro,
    operation: mapPersistedOperation(task.operation),
    status: active ? "failed" : mapPersistedStatus(task.status),
    phase: mapPersistedPhase(task.phase),
    percent: typeof task.percent === "number" ? task.percent : null,
    startedAt,
    endedAt: active ? new Date() : endedAt,
    error: active ? null : task.error,
    location: task.location,
    logoSrc: task.logoSrc,
    interrupted: active || task.interrupted,
  };
}

function mapPersistedOperation(operation: string): LongTaskOperation {
  if (
    operation === "install" ||
    operation === "importArchive" ||
    operation === "importVhd" ||
    operation === "export"
  ) {
    return operation;
  }

  return "export";
}

function mapPersistedStatus(status: string): LongTaskStatus {
  if (
    status === "started" ||
    status === "running" ||
    status === "completed" ||
    status === "failed"
  ) {
    return status;
  }

  return "failed";
}

function mapPersistedPhase(phase: string | null): LongTask["phase"] {
  if (
    phase === "Copying" ||
    phase === "Downloading" ||
    phase === "Installing" ||
    phase === "Importing" ||
    phase === "Exporting"
  ) {
    return phase;
  }

  return null;
}

function parsePersistedDate(value: string): Date {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? new Date() : date;
}

function toPersistedTask(task: LongTask): PersistedLongTask {
  return {
    requestId: task.requestId,
    distro: task.distro,
    operation: task.operation,
    status: task.status,
    phase: task.phase,
    percent: task.percent,
    startedAt: task.startedAt.toISOString(),
    endedAt: task.endedAt?.toISOString() ?? null,
    error: task.error,
    location: task.location,
    logoSrc: task.logoSrc,
    interrupted: task.interrupted,
  };
}

function persistLongTasks(tasks: LongTask[]): Promise<void> {
  if (!hasTauriBridge()) {
    return Promise.resolve();
  }

  persistQueue = persistQueue
    .catch(() => undefined)
    .then(() => saveLongTasks(tasks.map(toPersistedTask)));
  return persistQueue;
}

function persistLongTasksBestEffort(tasks: LongTask[]): void {
  if (!hasTauriBridge()) {
    return;
  }

  persistQueue = persistQueue
    .catch(() => undefined)
    .then(() => saveLongTasks(tasks.map(toPersistedTask)))
    .catch(() => undefined);
}
