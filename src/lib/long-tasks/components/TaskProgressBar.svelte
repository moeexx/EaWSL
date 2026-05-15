<script lang="ts">
  import { i18nState } from "$lib/i18n";

  import {
    getProgressFillStyle,
    getProgressPercent,
    getProgressStageLabel,
    isProgressIndeterminate,
  } from "../task-status";
  import type { LongTask } from "../state";

  type Props = {
    task: LongTask;
    className?: string;
  };

  let { task, className = "h-[7px] w-full" }: Props = $props();
  const copy = $derived($i18nState.copy.longTasks);
</script>

<div
  aria-label={`${task.distro} ${getProgressStageLabel(task, copy)}`}
  aria-valuemax="100"
  aria-valuemin="0"
  aria-valuenow={isProgressIndeterminate(task)
    ? undefined
    : getProgressPercent(task)}
  class={`${className} overflow-hidden rounded-full bg-shell-200/80`}
  role="progressbar"
>
  {#if isProgressIndeterminate(task)}
    <span
      aria-hidden="true"
      class="task-progress-indeterminate block h-full rounded-full"
    ></span>
  {:else}
    <span
      aria-hidden="true"
      class="block h-full rounded-full transition-[width] duration-500 ease-out"
      style={getProgressFillStyle(task)}
    ></span>
  {/if}
</div>

<style>
  .task-progress-indeterminate {
    width: 38%;
    background: linear-gradient(90deg, #1f7de6 0%, #4fa5f7 100%);
    animation: task-progress-slide 1.25s ease-in-out infinite;
  }

  @keyframes task-progress-slide {
    0% {
      transform: translateX(-120%);
    }

    100% {
      transform: translateX(285%);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .task-progress-indeterminate {
      animation: none;
    }
  }
</style>
