import AppWindow from "@lucide/svelte/icons/app-window";
import Cpu from "@lucide/svelte/icons/cpu";
import HardDrive from "@lucide/svelte/icons/hard-drive";
import MemoryStick from "@lucide/svelte/icons/memory-stick";
import Monitor from "@lucide/svelte/icons/monitor";

import { formatBytes, formatOptionalText } from "$lib/shared/format";
import type { AppCopy } from "$lib/i18n";
import type { SystemOverview } from "$lib/tauri/system";

export type OverviewSystemMetric = {
  label: string;
  value: string;
};

export type OverviewSystemInfoCard = {
  label: string;
  value: string;
  metrics: OverviewSystemMetric[];
  icon: typeof AppWindow;
  iconClass: string;
};

export function buildOverviewSystemCards(
  data: SystemOverview | null,
  isPending: boolean,
  copy: AppCopy,
): OverviewSystemInfoCard[] {
  const resolveValue = (value: string) =>
    isPending ? copy.common.loading : value;
  const labels = copy.overview.system.labels;

  return [
    {
      label: labels.windows,
      icon: AppWindow,
      iconClass: "bg-sky-50 text-sky-600",
      value: resolveValue(formatText(data?.windows.productName, copy)),
      metrics: [
        {
          label: labels.displayVersion,
          value: resolveValue(formatText(data?.windows.displayVersion, copy)),
        },
        {
          label: labels.buildNumber,
          value: resolveValue(formatText(data?.windows.buildNumber, copy)),
        },
      ],
    },
    {
      label: labels.cpu,
      icon: Cpu,
      iconClass: "bg-amber-50 text-amber-600",
      value: resolveValue(formatText(data?.cpu.model, copy)),
      metrics: [
        {
          label: labels.maxClock,
          value: resolveValue(formatClockMhz(data?.cpu.maxClockMhz, copy)),
        },
        {
          label: labels.cores,
          value: resolveValue(
            formatCoreSummary(
              data?.cpu.coreCount,
              data?.cpu.logicalProcessorCount,
              copy,
            ),
          ),
        },
      ],
    },
    {
      label: labels.memory,
      icon: MemoryStick,
      iconClass: "bg-emerald-50 text-emerald-600",
      value: resolveValue(
        formatBytes(data?.memory.totalBytes, 1, copy.common.missing),
      ),
      metrics: [
        {
          label: labels.speed,
          value: resolveValue(formatMemorySpeed(data?.memory.speedMts, copy)),
        },
        {
          label: labels.slots,
          value: resolveValue(
            formatSlotSummary(
              data?.memory.usedSlots,
              data?.memory.totalSlots,
              copy,
            ),
          ),
        },
      ],
    },
    {
      label: "GPU",
      icon: Monitor,
      iconClass: "bg-fuchsia-50 text-fuchsia-600",
      value: resolveValue(formatText(data?.gpu?.name, copy)),
      metrics: [
        {
          label: labels.gpuMemory,
          value: resolveValue(
            formatBytes(data?.gpu?.memoryBytes, 1, copy.common.missing),
          ),
        },
        {
          label: labels.driverVersion,
          value: resolveValue(formatText(data?.gpu?.driverVersion, copy)),
        },
      ],
    },
    {
      label: labels.storage,
      icon: HardDrive,
      iconClass: "bg-slate-100 text-slate-600",
      value: resolveValue(
        formatBytes(data?.storage.totalBytes, 2, copy.common.missing),
      ),
      metrics: [
        {
          label: labels.used,
          value: resolveValue(
            formatBytes(data?.storage.usedBytes, 2, copy.common.missing),
          ),
        },
        {
          label: labels.free,
          value: resolveValue(
            formatBytes(data?.storage.freeBytes, 2, copy.common.missing),
          ),
        },
        {
          label: labels.volumeCount,
          value: resolveValue(formatCount(data?.storage.volumeCount, copy)),
        },
      ],
    },
  ];
}

function formatText(value: string | null | undefined, copy: AppCopy): string {
  return formatOptionalText(value, { missingText: copy.common.missing });
}

function formatCount(value: number | null | undefined, copy: AppCopy): string {
  if (!Number.isFinite(value)) {
    return copy.common.missing;
  }

  return String(value);
}

function formatMemorySpeed(
  value: number | null | undefined,
  copy: AppCopy,
): string {
  if (!hasPositiveNumber(value)) {
    return copy.common.missing;
  }

  return `${value} MT/s`;
}

function formatClockMhz(
  value: number | null | undefined,
  copy: AppCopy,
): string {
  if (!hasPositiveNumber(value)) {
    return copy.common.missing;
  }

  if (value >= 1000) {
    return `${(value / 1000).toFixed(2)} GHz`;
  }

  return `${Math.round(value)} MHz`;
}

function formatCoreSummary(
  coreCount: number | null | undefined,
  logicalProcessorCount: number | null | undefined,
  copy: AppCopy,
): string {
  if (!hasPositiveNumber(coreCount)) {
    return copy.common.missing;
  }

  if (
    !hasPositiveNumber(logicalProcessorCount) ||
    logicalProcessorCount < coreCount
  ) {
    return copy.overview.system.coreCount(coreCount);
  }

  return copy.overview.system.coreThreadCount(coreCount, logicalProcessorCount);
}

function formatSlotSummary(
  usedSlots: number | null | undefined,
  totalSlots: number | null | undefined,
  copy: AppCopy,
): string {
  if (!hasNonNegativeNumber(usedSlots)) {
    return copy.common.missing;
  }

  if (!hasPositiveNumber(totalSlots)) {
    return copy.overview.system.usedSlots(usedSlots);
  }

  return copy.overview.system.slotUsage(usedSlots, totalSlots);
}

function hasPositiveNumber(value: number | null | undefined): value is number {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}

function hasNonNegativeNumber(
  value: number | null | undefined,
): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= 0;
}
