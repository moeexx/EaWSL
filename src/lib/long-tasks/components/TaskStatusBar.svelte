<script lang="ts">
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronUp from "@lucide/svelte/icons/chevron-up";

  import { i18nState } from "$lib/i18n";
  import Button from "$lib/ui/Button.svelte";
  import TaskProgressBar from "./TaskProgressBar.svelte";
  import TaskStatusCard from "./TaskStatusCard.svelte";
  import {
    getCollapsedActiveTaskHeader,
    getDotStyle,
    getProgressPercentLabel,
    getTaskStats,
    getTraySummaryText,
    getTrayTone,
  } from "../task-status";
  import { longTaskState } from "../state";
  import {
    setTaskTrayExpanded,
    shellUiState,
    toggleTaskTrayExpanded,
  } from "$lib/shell/state";

  const taskTrayPanelId = "task-status-tray-panel";
  let taskTrayElement: HTMLDivElement | undefined;
  const taskStats = $derived(getTaskStats($longTaskState.tasks));
  const copy = $derived($i18nState.copy.longTasks);
  const activeTask = $derived(taskStats.activeTask);
  const latestTask = $derived(taskStats.latestTask);
  const trayTone = $derived(getTrayTone(activeTask, latestTask));
  const collapsedActiveTaskHeader = $derived(
    activeTask
      ? getCollapsedActiveTaskHeader(activeTask, $i18nState.language, copy)
      : null,
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
  const trayButtonIcon = $derived(
    $shellUiState.taskTrayExpanded ? ChevronDown : ChevronUp,
  );
  const trayButtonLabel = $derived(
    $shellUiState.taskTrayExpanded
      ? copy.tray.collapseDetails
      : copy.tray.expandDetails,
  );

  const headerTitleClass =
    "min-w-0 shrink truncate text-[17px] font-semibold leading-[25px] text-shell-800";
  const headerMetaClass =
    "min-w-0 flex-1 truncate text-[14px] font-semibold leading-[25px]";

  $effect(() => {
    function handleDocumentPointerDown(event: PointerEvent): void {
      if (!$shellUiState.taskTrayExpanded || !taskTrayElement) {
        return;
      }

      if (
        event.target instanceof Node &&
        taskTrayElement.contains(event.target)
      ) {
        return;
      }

      setTaskTrayExpanded(false);
    }

    document.addEventListener("pointerdown", handleDocumentPointerDown, {
      capture: true,
    });

    return () => {
      document.removeEventListener("pointerdown", handleDocumentPointerDown, {
        capture: true,
      });
    };
  });
</script>

<div
  bind:this={taskTrayElement}
  class={`task-tray-surface pointer-events-auto mx-[20px] mb-[8px] text-[14px] text-secondary ${
    $shellUiState.taskTrayExpanded ? "task-tray-surface-expanded" : ""
  }`}
>
  <footer
    class={`relative flex h-[42px] min-h-[42px] items-center justify-between gap-3 overflow-hidden px-[18px] text-[15px] text-secondary ${
      $shellUiState.taskTrayExpanded ? "task-tray-header-expanded" : ""
    }`}
  >
    <div class="relative min-w-0 flex-1 self-stretch">
      <div
        class="task-tray-collapsed-content task-tray-header-content absolute inset-x-0 top-1/2 flex min-w-0 items-center gap-3"
        aria-hidden={$shellUiState.taskTrayExpanded}
      >
        <span
          aria-hidden="true"
          class={`h-[7px] w-[7px] shrink-0 rounded-full ${
            trayTone === "running" ? "animate-pulse" : ""
          }`}
          style={getDotStyle(trayTone)}
        ></span>
        {#if activeTask}
          <div class="flex min-w-0 flex-1 items-baseline gap-3">
            <strong
              class={headerTitleClass}
              title={collapsedActiveTaskHeader?.title}
            >
              {collapsedActiveTaskHeader?.title}
            </strong>
            <p
              class={`${headerMetaClass} text-shell-500`}
              title={collapsedActiveTaskHeader?.meta}
            >
              {collapsedActiveTaskHeader?.meta}
            </p>
          </div>

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
        {:else}
          <div class="flex min-w-0 flex-1 items-baseline gap-3">
            <strong class={headerTitleClass}>
              {copy.tray.barTitle}
            </strong>
            <p
              class={`${headerMetaClass} text-shell-400`}
              title={copy.tray.noRunningTasks}
            >
              {copy.tray.noRunningTasks}
            </p>
          </div>
        {/if}
      </div>

      <div
        class="task-tray-expanded-content task-tray-header-content absolute inset-x-0 top-1/2 flex min-w-0 items-center gap-3"
        aria-hidden={!$shellUiState.taskTrayExpanded}
      >
        <span
          aria-hidden="true"
          class={`h-[7px] w-[7px] shrink-0 rounded-full ${
            trayTone === "running" ? "animate-pulse" : ""
          }`}
          style={getDotStyle(trayTone)}
        ></span>
        {#if activeTask}
          <div class="flex min-w-0 flex-1 items-baseline gap-3">
            <strong class={headerTitleClass}>
              {copy.tray.barTitle}
            </strong>
            <p
              class={`${headerMetaClass} text-shell-500`}
              title={traySummaryText}
            >
              {traySummaryText}
            </p>
          </div>
        {:else}
          <div class="flex min-w-0 flex-1 items-baseline gap-3">
            <strong class={headerTitleClass}>
              {copy.tray.barTitle}
            </strong>
            <p
              class={`${headerMetaClass} text-shell-400`}
              title={copy.tray.noRunningTasks}
            >
              {copy.tray.noRunningTasks}
            </p>
          </div>
        {/if}
      </div>
    </div>

    <Button
      ariaControls={taskTrayPanelId}
      ariaExpanded={$shellUiState.taskTrayExpanded}
      ariaLabel={trayButtonLabel}
      icon={trayButtonIcon}
      variant="secondary"
      className="h-8 w-8 shrink-0 rounded-[8px] !border-transparent !bg-transparent px-0 text-shell-700 shadow-none hover:!border-transparent hover:!bg-white hover:text-accent-700 hover:opacity-100 focus-visible:ring-[#bddfff]"
      onclick={() => toggleTaskTrayExpanded()}
    />
  </footer>

  <div
    class={`task-tray-panel ${
      $shellUiState.taskTrayExpanded ? "task-tray-panel-expanded" : ""
    }`}
  >
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
</div>

<style>
  .border-tertiary {
    border-color: rgba(215, 225, 236, 0.92);
  }

  .text-secondary {
    color: #415165;
  }

  .task-tray-header-content {
    --task-tray-header-offset: 0px;
    transition:
      opacity 150ms ease,
      transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
    transform: translateY(calc(-50% + var(--task-tray-header-offset)));
  }

  .task-tray-collapsed-content {
    opacity: 1;
  }

  .task-tray-expanded-content {
    opacity: 0;
    pointer-events: none;
    --task-tray-header-offset: 5px;
  }

  .task-tray-header-expanded .task-tray-collapsed-content {
    opacity: 0;
    pointer-events: none;
    --task-tray-header-offset: -5px;
  }

  .task-tray-header-expanded .task-tray-expanded-content {
    opacity: 1;
    pointer-events: auto;
    --task-tray-header-offset: 0px;
  }

  .task-tray-surface {
    border: 0.5px solid #c9e5ff;
    border-radius: 8px;
    background: linear-gradient(180deg, #fafdff, #eff7ff);
    box-shadow: 0 5px 5px rgba(16, 26, 39, 0.08);
    overflow: hidden;
    transition:
      border-radius 220ms cubic-bezier(0.22, 1, 0.36, 1),
      background 220ms ease,
      box-shadow 220ms ease;
  }

  .task-tray-surface-expanded {
    border-color: #acd8ff;
    border-radius: 8px;
    background: linear-gradient(180deg, #f7fcff, #e8f3ff);
  }

  .task-tray-panel {
    max-height: 0;
    opacity: 0;
    overflow: hidden;
    transform: translateY(6px);
    transition:
      max-height 260ms cubic-bezier(0.22, 1, 0.36, 1),
      opacity 180ms ease,
      transform 220ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .task-tray-panel-expanded {
    max-height: 44vh;
    opacity: 1;
    transform: translateY(0);
  }

  @media (prefers-reduced-motion: reduce) {
    .task-tray-header-content,
    .task-tray-surface,
    .task-tray-panel {
      transition: none;
    }
  }
</style>
