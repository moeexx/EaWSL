import type { LongTask } from "./state";
import type { AppCopy, AppLanguage } from "$lib/i18n";
import { formatFullTime } from "$lib/shared/runtime";

type LongTasksCopy = AppCopy["longTasks"];

export type TrayTone = "idle" | "running" | "completed" | "failed";

export interface TaskStats {
  activeTask: LongTask | null;
  latestTask: LongTask | null;
  activeCount: number;
  completedCount: number;
  failedCount: number;
  totalCount: number;
}

const distroLogoByName: Record<string, string> = {
  debian: "/distro-logos/debian.svg",
  ubuntu: "/distro-logos/ubuntu.svg",
};

export function isTaskActive(task: LongTask): boolean {
  return task.status === "started" || task.status === "running";
}

export function clampProgressPercent(percent: number): number {
  return Math.max(0, Math.min(percent, 100));
}

export function getPhaseLabel(task: LongTask, copy: LongTasksCopy): string {
  if (task.phase === null) {
    return copy.phases.waiting;
  }

  switch (task.phase) {
    case "Copying":
      return copy.phases.Copying;
    case "Downloading":
      return copy.phases.Downloading;
    case "Installing":
      return copy.phases.Installing;
    case "Importing":
      return copy.phases.Importing;
    case "Exporting":
      return copy.phases.Exporting;
  }
}

export function getStatusLabel(
  status: LongTask["status"],
  copy: LongTasksCopy,
): string {
  if (status === "completed") {
    return copy.status.completed;
  }

  if (status === "failed") {
    return copy.status.failed;
  }

  return copy.status.running;
}

export function getDistroLogoSrc(distro: string): string {
  return distroLogoByName[distro.toLowerCase()] ?? "/distro-logos/generic.svg";
}

export function getProgressPercent(task: LongTask): number {
  if (task.status === "completed") {
    return 100;
  }

  if (task.percent !== null) {
    return clampProgressPercent(task.percent);
  }

  return task.status === "failed" ? 100 : 0;
}

export function isProgressIndeterminate(task: LongTask): boolean {
  return isTaskActive(task) && task.percent === null;
}

export function getProgressPercentLabel(task: LongTask): string | null {
  if (task.status === "completed") {
    return "100%";
  }

  if (task.percent !== null) {
    return `${clampProgressPercent(task.percent).toFixed(1)}%`;
  }

  return null;
}

export function getProgressStageLabel(
  task: LongTask,
  copy: LongTasksCopy,
): string {
  if (task.interrupted) {
    return copy.progress.interrupted;
  }

  if (task.status === "completed") {
    return copy.progress.completed;
  }

  if (task.status === "failed") {
    return copy.progress.failed;
  }

  return getPhaseLabel(task, copy);
}

export function getTaskErrorMessage(
  task: LongTask,
  copy: LongTasksCopy,
): string | null {
  if (task.interrupted) {
    return copy.errors.interrupted;
  }

  return task.error;
}

export function getOperationLabel(task: LongTask, copy: LongTasksCopy): string {
  if (task.operation === "export") {
    return copy.operations.export;
  }

  if (task.operation === "importArchive") {
    return copy.operations.importArchive;
  }

  if (task.operation === "importVhd") {
    return copy.operations.importVhd;
  }

  return copy.operations.install;
}

export function getProgressFillStyle(task: LongTask): string {
  const background =
    task.status === "failed"
      ? "linear-gradient(90deg, #dc2626 0%, #ef4444 100%)"
      : task.status === "completed"
        ? "linear-gradient(90deg, #2f7d32 0%, #5fbf68 100%)"
        : "linear-gradient(90deg, #1f7de6 0%, #4fa5f7 100%)";

  return `width: ${getProgressPercent(task)}%; background: ${background};`;
}

export function getTaskCardClass(task: LongTask): string {
  const base =
    "grid gap-3 rounded-[8px] border-[0.5px] bg-white px-3.5 py-3 shadow-[0_6px_16px_rgba(15,23,42,0.035)]";

  if (task.status === "failed") {
    return `${base} border-rose-200/85`;
  }

  if (task.status === "completed") {
    return `${base} border-emerald-200/80`;
  }

  return `${base} border-shell-200/90`;
}

export function getTaskMetaItems(
  task: LongTask,
  language: AppLanguage,
  copy: LongTasksCopy,
): Array<{ label: string; value: string }> {
  const items = [
    {
      label: copy.card.distro,
      value: task.distro,
    },
    {
      label: copy.card.startedAt,
      value: formatFullTime(task.startedAt, language),
    },
  ];

  if (task.endedAt !== null) {
    items.push({
      label: task.status === "failed" ? copy.card.failedAt : copy.card.endedAt,
      value: formatFullTime(task.endedAt, language),
    });
  }

  return items;
}

export function getTrayTone(
  active: LongTask | null,
  latest: LongTask | null,
): TrayTone {
  if (active !== null) {
    return "running";
  }

  if (latest?.status === "failed") {
    return "failed";
  }

  return latest === null ? "idle" : "completed";
}

export function getDotStyle(tone: TrayTone): string {
  if (tone === "idle") {
    return "background-color: transparent; border: 1.5px solid #888780;";
  }

  const fill =
    tone === "running" ? "#378ADD" : tone === "failed" ? "#DC2626" : "#3B6D11";
  return `background-color: ${fill}; border: 1.5px solid ${fill};`;
}

export function getStatusBadgeClass(status: LongTask["status"]): string {
  const base =
    "inline-flex h-[22px] shrink-0 items-center rounded-[6px] border-[0.5px] px-2 text-[12px] font-semibold leading-none";

  if (status === "completed") {
    return `${base} border-[#bfd8bc] bg-[#eef8ee] text-[#2f6f2d]`;
  }

  if (status === "failed") {
    return `${base} border-[#fecaca] bg-[#fff1f2] text-[#b91c1c]`;
  }

  return `${base} border-accent-200 bg-accent-50 text-accent-700`;
}

export function getCollapsedTaskStatus(
  task: LongTask,
  copy: LongTasksCopy,
): string {
  const operation = getOperationLabel(task, copy);

  if (task.status === "failed") {
    return copy.collapsed.failed(operation);
  }

  if (task.status === "completed") {
    return copy.collapsed.completed(operation);
  }

  return getPhaseLabel(task, copy);
}

export function getCollapsedTaskMeta(
  task: LongTask,
  isCurrentTask: boolean,
  language: AppLanguage,
  copy: LongTasksCopy,
): string {
  const prefix = isCurrentTask
    ? copy.tray.currentTask
    : copy.tray.recentTask;

  if (task.status === "failed" && task.endedAt !== null) {
    return `${prefix} · ${copy.tray.failedAt}${formatFullTime(task.endedAt, language)}`;
  }

  if (task.status === "completed" && task.endedAt !== null) {
    return `${prefix} · ${copy.tray.endedAt}${formatFullTime(task.endedAt, language)}`;
  }

  return `${prefix} · ${copy.tray.startedAt}${formatFullTime(task.startedAt, language)}`;
}

export function getTraySummaryText(
  total: number,
  active: number,
  completed: number,
  failed: number,
  copy: LongTasksCopy,
): string {
  if (total === 0) {
    return copy.tray.noTasks;
  }

  const parts = [copy.tray.total(total, copy.tray.totalTasks)];

  if (active > 0) {
    parts.push(copy.tray.active(active));
  }

  if (completed > 0) {
    parts.push(copy.tray.completed(completed));
  }

  if (failed > 0) {
    parts.push(copy.tray.failed(failed));
  }

  return parts.join(" · ");
}

export function getTaskStats(tasks: LongTask[]): TaskStats {
  let activeTask: LongTask | null = null;
  let activeCount = 0;
  let completedCount = 0;
  let failedCount = 0;

  for (const task of tasks) {
    if (isTaskActive(task)) {
      activeCount += 1;
      activeTask ??= task;
    } else if (task.status === "completed") {
      completedCount += 1;
    } else if (task.status === "failed") {
      failedCount += 1;
    }
  }

  return {
    activeTask,
    latestTask: activeTask ?? tasks[0] ?? null,
    activeCount,
    completedCount,
    failedCount,
    totalCount: tasks.length,
  };
}
