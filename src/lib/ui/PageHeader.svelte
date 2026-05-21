<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    eyebrow?: string;
    title: string;
    description?: string;
    actions?: Snippet;
  };

  let {
    eyebrow = "Workspace",
    title,
    description = undefined,
    actions,
  }: Props = $props();
</script>

<header
  class="relative isolate overflow-hidden rounded-[8px] border-[0.5px] border-shell-200/80 bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(247,250,254,0.92))] px-4 py-[9px] shadow-[0_12px_32px_rgba(17,26,39,0.07)]"
>
  <div
    aria-hidden="true"
    class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(79,165,247,0.18),transparent_34%),radial-gradient(circle_at_bottom_right,rgba(32,123,229,0.08),transparent_38%),linear-gradient(180deg,rgba(255,255,255,0.98),rgba(246,249,253,0.9))]"
  ></div>

  <div
    aria-hidden="true"
    class="pointer-events-none absolute left-0 top-0 h-full w-[3px] bg-[linear-gradient(180deg,rgba(32,123,229,0.78),rgba(79,165,247,0.32),transparent)]"
  ></div>

  <div
    class="relative flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between sm:gap-8"
  >
    <div class="flex min-w-0 flex-col items-start gap-1">
      <div
        class="inline-flex items-center gap-1.5 rounded-full border-[0.5px] border-accent-100 bg-accent-50/80 px-2.5 py-0.5 text-[9px] font-semibold uppercase tracking-[0.2em] text-accent-700"
      >
        <span class="h-1 w-1 rounded-full bg-accent-500"></span>
        <span>{eyebrow}</span>
      </div>

      <h1
        class="text-[1.55rem] font-semibold tracking-[-0.04em] text-shell-950"
      >
        {title}
      </h1>

      {#if description}
        <p class="w-full text-[13px] leading-5 text-shell-600">
          {description}
        </p>
      {/if}
    </div>

    {#if actions}
      <div class="shrink-0">
        <div
          class="flex flex-wrap items-center justify-start gap-1.5 sm:justify-end"
        >
          {@render actions()}
        </div>
      </div>
    {/if}
  </div>
</header>
