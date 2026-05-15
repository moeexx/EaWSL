import { writable } from "svelte/store";

export type ConfirmDialogTone = "warning" | "danger" | "info";
export type ConfirmDialogActionVariant = "primary" | "danger" | "secondary";

export interface ConfirmDialogAction {
  id: string;
  label: string;
  variant: ConfirmDialogActionVariant;
}

export interface ConfirmDialogOptions {
  title: string;
  message: string;
  tone: ConfirmDialogTone;
  actions: ConfirmDialogAction[];
}

interface ActiveConfirmDialog extends ConfirmDialogOptions {
  resolve: (actionId: string) => void;
}

const store = writable<ActiveConfirmDialog | null>(null);

let activeDialog: ActiveConfirmDialog | null = null;

export const confirmDialogState = {
  subscribe: store.subscribe,
};

function clearDialog(): void {
  activeDialog = null;
  store.set(null);
}

export function resolveConfirmDialog(actionId: string): void {
  const dialog = activeDialog;

  if (!dialog) {
    return;
  }

  clearDialog();
  dialog.resolve(actionId);
}

export function openConfirmDialog(
  options: ConfirmDialogOptions,
): Promise<string> {
  if (activeDialog) {
    return Promise.resolve("cancel");
  }

  return new Promise<string>((resolve) => {
    const dialog: ActiveConfirmDialog = {
      ...options,
      resolve,
    };

    activeDialog = dialog;
    store.set(dialog);
  });
}
