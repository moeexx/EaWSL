import { writable } from "svelte/store";

import { createRequestId } from "$lib/shared/runtime";

export type ToastTone = "success" | "error" | "warning" | "info";

export interface ToastItem {
  id: string;
  tone: ToastTone;
  title: string;
  message: string;
}

export interface ToastInput {
  tone: ToastTone;
  title: string;
  message: string;
  durationMs?: number;
}

const DEFAULT_TOAST_DURATION_MS = 4200;

const store = writable<ToastItem[]>([]);
const timers = new Map<string, ReturnType<typeof setTimeout>>();

export const toastState = {
  subscribe: store.subscribe,
};

export function dismissToast(id: string): void {
  const timer = timers.get(id);

  if (timer) {
    clearTimeout(timer);
    timers.delete(id);
  }

  store.update((items) => items.filter((item) => item.id !== id));
}

export function pushToast(input: ToastInput): string {
  const id = createRequestId();
  const durationMs = input.durationMs ?? DEFAULT_TOAST_DURATION_MS;
  const toast: ToastItem = {
    id,
    tone: input.tone,
    title: input.title,
    message: input.message,
  };

  store.update((items) => [toast, ...items]);

  timers.set(
    id,
    setTimeout(() => {
      dismissToast(id);
    }, durationMs),
  );

  return id;
}
