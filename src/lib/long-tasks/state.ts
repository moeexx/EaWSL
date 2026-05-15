import { writable } from "svelte/store";

import {
  listenTransferProgress,
  type DistroProgressEvent,
  type ProgressPhase,
  type ProgressState,
  type ProgressValue,
} from "$lib/tauri/wsl";
import { hasTauriBridge } from "$lib/shared/runtime";

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
}

export interface StartTaskInput {
  requestId: string;
  distro: string;
  operation: LongTaskOperation;
  location: string | null;
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

export const longTaskState = {
  subscribe: store.subscribe,
};

function syncActiveFlag(tasks: LongTask[]): LongTaskState {
  return {
    tasks,
    hasActiveLongTask: tasks.some((task) => task.status === "started" || task.status === "running"),
  };
}

function updateTask(requestId: string, updater: (task: LongTask) => LongTask): void {
  store.update((state) =>
    syncActiveFlag(state.tasks.map((task) => (task.requestId === requestId ? updater(task) : task))),
  );
}

function extractProgressState(value: ProgressValue): ProgressState | null {
  return "Status" in value ? value.Status : null;
}

function extractProgressPercent(value: ProgressValue): number | null {
  return "Percent" in value ? value.Percent : null;
}

export function startTask(input: StartTaskInput): void {
  store.update((state) =>
    syncActiveFlag([
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
      },
      ...state.tasks,
    ]),
  );
}

export function failTask(requestId: string, error: string): void {
  updateTask(requestId, (task) => ({
    ...task,
    status: "failed",
    endedAt: task.endedAt ?? new Date(),
    error,
  }));
}

export function completeTask(requestId: string): void {
  updateTask(requestId, (task) => ({
    ...task,
    status: "completed",
    endedAt: task.endedAt ?? new Date(),
    error: null,
  }));
}

export function applyLongTaskProgress(
  payload: DistroProgressEvent,
): void {
  const progressState = extractProgressState(payload.progress.value);
  const progressPercent = extractProgressPercent(payload.progress.value);

  updateTask(payload.requestId, (task) => {
    const phaseChanged = task.phase !== payload.progress.phase;
    const nextTask: LongTask = {
      ...task,
      phase: payload.progress.phase,
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
