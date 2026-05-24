<script lang="ts">
  import "../app.css";

  import type { Snippet } from "svelte";

  import ShellSidebar from "$lib/shell/ShellSidebar.svelte";
  import ConfirmDialogViewport from "$lib/feedback/ConfirmDialogViewport.svelte";
  import ShellIcon from "$lib/shell/ShellIcon.svelte";
  import TaskStatusBar from "$lib/long-tasks/components/TaskStatusBar.svelte";
  import ToastViewport from "$lib/feedback/ToastViewport.svelte";
  import { i18nState, startI18n } from "$lib/i18n";
  import { startLongTaskFeed, stopLongTaskFeed } from "$lib/long-tasks";
  import { startQueryCache, stopQueryCache } from "$lib/query-cache";
  import {
    closeWindow,
    minimizeWindow,
    shellUiState,
    startResizeDrag,
    startShellUi,
    stopShellUi,
    toggleMaximizeWindow,
  } from "$lib/shell/state";

  let { children }: { children?: Snippet } = $props();
  let taskTrayInset = $state(0);
  let taskTrayWrapperElement: HTMLDivElement | undefined;

  const resizeHandles = [
    {
      direction: "North",
      classes: "absolute left-6 right-6 top-0 h-2 cursor-ns-resize",
    },
    {
      direction: "South",
      classes: "absolute bottom-0 left-6 right-6 h-2 cursor-ns-resize",
    },
    {
      direction: "East",
      classes: "absolute right-0 top-6 bottom-6 w-2 cursor-ew-resize",
    },
    {
      direction: "West",
      classes: "absolute left-0 top-6 bottom-6 w-2 cursor-ew-resize",
    },
    {
      direction: "NorthEast",
      classes: "absolute right-0 top-0 h-3 w-3 cursor-nesw-resize",
    },
    {
      direction: "NorthWest",
      classes: "absolute left-0 top-0 h-3 w-3 cursor-nwse-resize",
    },
    {
      direction: "SouthEast",
      classes: "absolute bottom-0 right-0 h-3 w-3 cursor-nwse-resize",
    },
    {
      direction: "SouthWest",
      classes: "absolute bottom-0 left-0 h-3 w-3 cursor-nesw-resize",
    },
  ] as const;

  const titlebarButtonClass =
    "group flex w-10 items-center justify-center border-0 bg-transparent text-shell-600 transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-300 hover:bg-accent-50 hover:text-accent-700";

  const closeButtonClass =
    "group flex w-10 items-center justify-center border-0 bg-transparent text-shell-600 transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-300 hover:bg-red-600 hover:text-white";

  const canResizeWindow = $derived(!$shellUiState.windowMaximized);
  const shellCopy = $derived($i18nState.copy.shell);

  $effect(() => {
    startI18n();
    startQueryCache();
    startLongTaskFeed();
    startShellUi();

    return () => {
      stopShellUi();
      stopLongTaskFeed();
      stopQueryCache();
    };
  });

  $effect(() => {
    if (typeof ResizeObserver === "undefined" || !taskTrayWrapperElement) {
      syncTaskTrayInset();
      return;
    }

    const observer = new ResizeObserver(() => {
      syncTaskTrayInset();
    });

    observer.observe(taskTrayWrapperElement);
    syncTaskTrayInset();

    return () => {
      observer.disconnect();
    };
  });

  function syncTaskTrayInset(): void {
    taskTrayInset = taskTrayWrapperElement
      ? Math.ceil(taskTrayWrapperElement.getBoundingClientRect().height)
      : 0;
  }
</script>

<div class="relative h-screen overflow-hidden p-0 text-shell-900">
  {#if canResizeWindow}
    {#each resizeHandles as handle}
      <button
        aria-label={shellCopy.titlebar.resize(
          shellCopy.titlebar.resizeDirections[handle.direction],
        )}
        class={`z-30 border-0 bg-transparent p-0 ${handle.classes}`}
        onmousedown={() => void startResizeDrag(handle.direction)}
        type="button"
      ></button>
    {/each}
  {/if}

  <div class="relative flex h-full min-h-0 flex-col overflow-hidden">
    <header class="relative z-10 flex h-10 items-stretch justify-between">
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="flex min-w-0 flex-1 items-center justify-between gap-2.5 px-3 sm:px-3.5"
        data-tauri-drag-region
        ondblclick={() => void toggleMaximizeWindow()}
      >
        <div class="flex min-w-0 items-center gap-2.5" data-tauri-drag-region>
          <div
            class="flex h-[45px] w-[45px] shrink-0 translate-y-[5px] items-center justify-center rounded-[8px]"
            data-tauri-drag-region
          >
            <img
              alt=""
              aria-hidden="true"
              class="h-[45px] w-[45px] rounded-[8px] object-contain"
              src="/favicon.png"
            />
          </div>

          <div class="min-w-0" data-tauri-drag-region>
            <strong
              class="block truncate text-sm font-semibold text-shell-950"
              data-tauri-drag-region
            >
              EaWSL
            </strong>
          </div>
        </div>
      </div>

      <div class="flex shrink-0 items-stretch">
        <button
          aria-label={shellCopy.titlebar.minimize}
          class={titlebarButtonClass}
          onclick={() => void minimizeWindow()}
          type="button"
        >
          <ShellIcon name="minimize" size={14} />
        </button>

        <button
          aria-label={$shellUiState.windowMaximized
            ? shellCopy.titlebar.restore
            : shellCopy.titlebar.maximize}
          class={titlebarButtonClass}
          onclick={() => void toggleMaximizeWindow()}
          type="button"
        >
          <ShellIcon
            name={$shellUiState.windowMaximized ? "restore" : "maximize"}
            size={14}
          />
        </button>

        <button
          aria-label={shellCopy.titlebar.close}
          class={closeButtonClass}
          onclick={() => void closeWindow()}
          type="button"
        >
          <ShellIcon name="close" size={18} />
        </button>
      </div>
    </header>

    <div class="relative flex min-h-0 flex-1">
      <ShellSidebar />
      <main class="relative min-w-0 flex-1 overflow-hidden">
        <div
          class="ui-scrollbar h-full overflow-auto py-[12px] pl-[12px] pr-[2px]"
          style={`padding-bottom: ${12 + taskTrayInset}px;`}
        >
          {@render children?.()}
        </div>

        <div
          bind:this={taskTrayWrapperElement}
          class="pointer-events-none absolute inset-x-0 bottom-0 z-20 mx-[12px] mb-[8px]"
        >
          <TaskStatusBar />
        </div>
      </main>
    </div>

    <ToastViewport />
    <ConfirmDialogViewport />
  </div>
</div>
