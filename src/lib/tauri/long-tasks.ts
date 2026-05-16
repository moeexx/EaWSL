import { invoke } from "@tauri-apps/api/core";

import { normalizeTauriCommandError } from "./errors";

export interface PersistedLongTask {
  requestId: string;
  distro: string;
  operation: string;
  status: string;
  phase: string | null;
  percent: number | null;
  startedAt: string;
  endedAt: string | null;
  error: string | null;
  location: string | null;
  interrupted: boolean;
}

async function invokeLongTasks<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeTauriCommandError(error);
  }
}

export function getLongTasks(): Promise<PersistedLongTask[]> {
  return invokeLongTasks("get_long_tasks");
}

export function saveLongTasks(tasks: PersistedLongTask[]): Promise<void> {
  return invokeLongTasks("save_long_tasks", { tasks });
}
