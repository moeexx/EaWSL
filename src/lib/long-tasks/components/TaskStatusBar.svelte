<script lang="ts">
  import ShellIcon from "$lib/shell/ShellIcon.svelte";
  import { i18nState } from "$lib/i18n";
  import TaskProgressBar from "./TaskProgressBar.svelte";
  import TaskStatusCard from "./TaskStatusCard.svelte";
  import {
    getCollapsedTaskMeta,
    getCollapsedTaskStatus,
    getDotStyle,
    getPhaseLabel,
    getProgressPercentLabel,
    getTaskStats,
    getTraySummaryText,
    getTrayTone,
  } from "../task-status";
  import { longTaskState } from "../state";
  import { shellUiState, toggleTaskTrayExpanded } from "$lib/shell/state";

  const taskTrayPanelId = "task-status-tray-panel";
  type TrayButtonIcon = "chevron-up" | "chevron-down";

  const taskStats = $derived(getTaskStats($longTaskState.tasks));
  const copy = $derived($i18nState.copy.longTasks);
  const activeTask = $derived(taskStats.activeTask);
  const latestTask = $derived(taskStats.latestTask);
  const trayTone = $derived(getTrayTone(activeTask, latestTask));
  const collapsedStatusTitle = $derived(
    latestTask
      ? `${latestTask.distro} · ${getCollapsedTaskStatus(latestTask, copy)}`
      : copy.tray.noTasks,
  );
  const collapsedStatusMeta = $derived(
    latestTask
      ? getCollapsedTaskMeta(
          latestTask,
          activeTask !== null,
          $i18nState.language,
          copy,
        )
      : null,
  );
  const collapsedStatusText = $derived(
    collapsedStatusMeta
      ? `${collapsedStatusTitle} · ${collapsedStatusMeta}`
      : collapsedStatusTitle,
  );
  const activePhaseLabel = $derived(
    activeTask ? getPhaseLabel(activeTask, copy) : null,
  );
  const traySummaryText = $derived(
    getTraySummaryText(
      taskStats.totalCount,
      taskStats.activeCount,
      taskStats.completedCount,
      taskStats.failedCount,
      copy,
    ),
  );
  const trayButtonIcon: TrayButtonIcon = $derived(
    $shellUiState.taskTrayExpanded ? "chevron-down" : "chevron-up",
  );
</script>

<section
  class="pointer-events-none relative z-10 w-full text-[14px] text-secondary"
>
  <div
    class={`task-status-drawer pointer-events-auto rounded-t-[8px] background-secondary text-secondary ${
      $shellUiState.taskTrayExpanded ? "task-status-drawer-expanded" : ""
    }`}
  >
    <footer
      class={`relative flex h-[var(--task-tray-collapsed-height)] min-h-[var(--task-tray-collapsed-height)] items-center justify-between gap-3 overflow-hidden rounded-t-[8px] border-t-[0.5px] border-tertiary background-secondary px-[14px] text-[15px] text-secondary ${
        $shellUiState.taskTrayExpanded ? "task-tray-header-expanded" : ""
      }`}
    >
      <div class="relative min-w-0 flex-1 self-stretch">
        <div
          class="task-tray-collapsed-content task-tray-header-content absolute inset-0 flex min-w-0 items-center gap-2.5"
          aria-hidden={$shellUiState.taskTrayExpanded}
        >
          <span
            aria-hidden="true"
            class={`h-[7px] w-[7px] shrink-0 rounded-full ${
              trayTone === "running" ? "animate-pulse" : ""
            }`}
            style={getDotStyle(trayTone)}
          ></span>
          <div class="min-w-0 flex-1">
            <strong
              class="block min-h-[24px] truncate text-[17px] font-semibold leading-[1.4] text-shell-800"
              title={collapsedStatusText}
            >
              {collapsedStatusTitle}
            </strong>
            {#if collapsedStatusMeta}
              <p
                class="mt-0.5 truncate text-[13px] leading-4 text-secondary"
                style="opacity: 0.82;"
                title={collapsedStatusMeta}
              >
                {collapsedStatusMeta}
              </p>
            {/if}
          </div>
          {#if activePhaseLabel}
            <span
              class="hidden h-[22px] shrink-0 items-center rounded-[6px] border-[0.5px] border-tertiary background-primary px-2 text-[12px] leading-none text-secondary md:inline-flex"
              style="opacity: 0.86;"
            >
              {activePhaseLabel}
            </span>
          {/if}

          {#if activeTask}
            {@const progressPercentLabel = getProgressPercentLabel(activeTask)}
            <div
              class="flex min-w-[96px] max-w-[230px] flex-[0_1_190px] items-center gap-2"
            >
              <TaskProgressBar
                task={activeTask}
                className="h-[7px] min-w-0 flex-1"
              />
              {#if progressPercentLabel}
                <span
                  class="w-[48px] shrink-0 whitespace-nowrap text-right text-[13px] font-medium leading-none text-shell-700"
                >
                  {progressPercentLabel}
                </span>
              {/if}
            </div>
          {/if}
        </div>

        <div
          class="task-tray-expanded-content task-tray-header-content absolute inset-0 flex min-w-0 items-center gap-2.5"
          aria-hidden={!$shellUiState.taskTrayExpanded}
        >
          <span
            class="flex h-8 w-8 shrink-0 items-center justify-center rounded-[8px] border-[0.5px] border-tertiary background-primary text-accent-700"
          >
            <ShellIcon name="tasks" size={17} />
          </span>
          <div class="min-w-0">
            <strong
              class="block truncate text-[14px] font-semibold leading-5 text-shell-800"
            >
              {copy.tray.title}
            </strong>
            <p class="truncate text-[13px] leading-4 text-secondary">
              {traySummaryText}
            </p>
          </div>
        </div>
      </div>

      <button
        aria-controls={taskTrayPanelId}
        aria-expanded={$shellUiState.taskTrayExpanded}
        class="inline-flex h-[30px] shrink-0 items-center justify-center gap-1.5 rounded-[7px] border-[0.5px] border-tertiary background-primary px-2.5 text-[13px] font-semibold leading-none text-secondary transition hover:opacity-95 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#bddfff]"
        onclick={() => toggleTaskTrayExpanded()}
        type="button"
      >
        <ShellIcon name={trayButtonIcon} size={14} />
        <span>
          {$shellUiState.taskTrayExpanded
            ? copy.tray.collapseDetails
            : copy.tray.expandDetails}
        </span>
      </button>
    </footer>

    <div
      id={taskTrayPanelId}
      aria-hidden={!$shellUiState.taskTrayExpanded}
      aria-label={copy.tray.title}
      class="ui-scrollbar max-h-[44vh] overflow-auto border-t-[0.5px] border-tertiary px-[14px] py-3"
      role="region"
    >
      {#if taskStats.totalCount === 0}
        <div
          class="py-5 text-[14px] leading-5 text-secondary"
          style="opacity: 0.8;"
        >
          {copy.tray.noTasks}
        </div>
      {:else}
        <ul class="grid gap-2.5">
          {#each $longTaskState.tasks as task (task.requestId)}
            <TaskStatusCard {task} />
          {/each}
        </ul>
      {/if}
    </div>
  </div>
</section>

<style>
  .background-primary {
    background-color: rgba(255, 255, 255, 0.92);
  }

  .background-secondary {
    background: linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.98),
      rgba(247, 250, 253, 0.98)
    );
  }

  .border-tertiary {
    border-color: rgba(215, 225, 236, 0.92);
  }

  .text-secondary {
    color: #415165;
  }

  .task-tray-header-content {
    transition:
      opacity 150ms ease,
      transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .task-tray-collapsed-content {
    opacity: 1;
    transform: translateY(0);
  }

  .task-tray-expanded-content {
    opacity: 0;
    pointer-events: none;
    transform: translateY(5px);
  }

  .task-tray-header-expanded .task-tray-collapsed-content {
    opacity: 0;
    pointer-events: none;
    transform: translateY(-5px);
  }

  .task-tray-header-expanded .task-tray-expanded-content {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0);
  }

  .task-status-drawer {
    box-shadow: 0 -3px 8px rgba(16, 26, 39, 0.2);
    transform: translateY(calc(100% - var(--task-tray-collapsed-height)));
    transition: transform 320ms cubic-bezier(0.22, 1, 0.36, 1);
    will-change: transform;
  }

  .task-status-drawer-expanded {
    transform: translateY(0);
  }

  @media (prefers-reduced-motion: reduce) {
    .task-tray-header-content,
    .task-status-drawer {
      transition: none;
    }
  }
</style>
