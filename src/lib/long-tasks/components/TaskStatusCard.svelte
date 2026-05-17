<script lang="ts">
  import { i18nState } from "$lib/i18n";
  import DistroLogo from "$lib/ui/DistroLogo.svelte";

  import {
    getOperationLabel,
    getProgressPercentLabel,
    getProgressStageLabel,
    getStatusBadgeClass,
    getStatusLabel,
    getTaskCardClass,
    getTaskErrorMessage,
    getTaskMetaItems,
  } from "../task-status";
  import type { LongTask } from "../state";

  import TaskProgressBar from "./TaskProgressBar.svelte";

  type Props = { task: LongTask };

  let { task }: Props = $props();
  const copy = $derived($i18nState.copy.longTasks);
  const progressPercentLabel = $derived(getProgressPercentLabel(task));
  const taskErrorMessage = $derived(getTaskErrorMessage(task, copy));
</script>

<li class={getTaskCardClass(task)}>
  <div class="flex min-w-0 items-start justify-between gap-3">
    <div class="flex min-w-0 items-start gap-3">
      <DistroLogo alt={copy.card.logoAlt(task.distro)} src={task.logoSrc} />

      <div class="min-w-0 flex-1">
        <div class="flex min-w-0 flex-wrap items-center gap-2">
          <strong
            class="min-w-0 truncate text-[16px] font-semibold leading-5 text-shell-900"
          >
            {task.distro}
          </strong>
          <span
            class="inline-flex h-[22px] shrink-0 items-center rounded-[6px] border-[0.5px] border-tertiary bg-shell-50/80 px-2 text-[12px] font-semibold leading-none text-secondary"
          >
            {getOperationLabel(task, copy)}
          </span>
        </div>
        {#if task.location}
          <p
            class="mt-1.5 truncate text-[12.5px] leading-5 text-shell-600"
            title={task.location}
          >
            {copy.card.location}: {task.location}
          </p>
        {/if}
      </div>
    </div>

    <div
      class="flex min-w-0 max-w-[46%] shrink-0 flex-col items-end gap-1.5 text-right"
    >
      <span class={getStatusBadgeClass(task.status)}>
        {getStatusLabel(task.status, copy)}
      </span>
    </div>
  </div>

  <div
    class="grid gap-3 md:grid-cols-[minmax(0,1fr)_310px] md:items-end min-[1500px]:grid-cols-[minmax(0,1fr)_470px]"
  >
    <dl class="grid min-w-0 grid-cols-2 gap-x-4 gap-y-2 md:grid-cols-3">
      {#each getTaskMetaItems(task, $i18nState.language, copy) as item}
        <div class="min-w-0">
          <dt class="text-[11px] font-medium leading-4 text-shell-500">
            {item.label}
          </dt>
          <dd
            class="truncate text-[12.5px] font-medium leading-5 text-shell-800"
            title={item.value}
          >
            {item.value}
          </dd>
        </div>
      {/each}
    </dl>

    <div
      class="task-card-progress-group grid gap-1 justify-self-start md:justify-self-end"
    >
      <div
        class="flex w-full items-center justify-between gap-2 text-[12px] leading-4 text-secondary"
        style="opacity: 0.78;"
      >
        <span class="min-w-0 truncate">
          {getProgressStageLabel(task, copy)}
        </span>
        {#if progressPercentLabel}
          <span
            class="shrink-0 whitespace-nowrap text-[12px] font-semibold text-shell-700"
          >
            {progressPercentLabel}
          </span>
        {/if}
      </div>

      <TaskProgressBar {task} />
    </div>
  </div>

  {#if task.status === "failed" && taskErrorMessage}
    <div
      class="rounded-[7px] border-[0.5px] border-rose-200/80 bg-rose-50/80 px-3 py-2 text-[12.5px] font-medium leading-5 text-rose-700"
    >
      {taskErrorMessage}
    </div>
  {/if}
</li>

<style>
  .border-tertiary {
    border-color: rgba(215, 225, 236, 0.92);
  }

  .text-secondary {
    color: #415165;
  }

  .task-card-progress-group {
    max-width: 100%;
    width: 300px;
  }

  @media (min-width: 1500px) {
    .task-card-progress-group {
      width: 460px;
    }
  }
</style>
