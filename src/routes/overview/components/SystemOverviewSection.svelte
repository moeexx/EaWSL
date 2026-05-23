<script lang="ts">
  import SectionPanel from "$lib/ui/SectionPanel.svelte";
  import type { AppCopy } from "$lib/i18n";

  import type { OverviewSystemInfoCard } from "../view-model/system-overview-cards";

  type Props = {
    cards: OverviewSystemInfoCard[];
    copy: AppCopy["overview"];
  };

  let { cards, copy }: Props = $props();
  let headerElements: Array<HTMLElement | null> = [];
  let syncFrameId: number | null = null;

  $effect(() => {
    cards;

    if (typeof ResizeObserver === "undefined") {
      return;
    }

    const elements = headerElements.filter(
      (element): element is HTMLElement => element !== null,
    );

    if (elements.length === 0) {
      return;
    }

    const observer = new ResizeObserver(() => {
      scheduleHeaderSync();
    });

    for (const element of elements) {
      observer.observe(element);
    }

    scheduleHeaderSync();

    return () => {
      observer.disconnect();

      if (syncFrameId !== null) {
        cancelAnimationFrame(syncFrameId);
        syncFrameId = null;
      }
    };
  });

  function scheduleHeaderSync(): void {
    if (syncFrameId !== null) {
      cancelAnimationFrame(syncFrameId);
    }

    syncFrameId = requestAnimationFrame(() => {
      syncFrameId = null;
      syncHeaderHeights();
    });
  }

  function syncHeaderHeights(): void {
    const elements = headerElements.filter(
      (element): element is HTMLElement => element !== null,
    );

    if (elements.length === 0) {
      return;
    }

    for (const element of elements) {
      element.style.height = "";
    }

    const rows = new Map<number, HTMLElement[]>();
    const rowHeights = new Map<number, number>();

    for (const element of elements) {
      const rect = element.getBoundingClientRect();
      const rowTop = Math.round(rect.top);
      const naturalHeight = Math.ceil(rect.height);
      const rowElements = rows.get(rowTop) ?? [];

      rowElements.push(element);
      rows.set(rowTop, rowElements);

      if (naturalHeight > (rowHeights.get(rowTop) ?? 0)) {
        rowHeights.set(rowTop, naturalHeight);
      }
    }

    for (const [rowTop, rowElements] of rows) {
      const targetHeight = rowHeights.get(rowTop);

      if (!targetHeight) {
        continue;
      }

      const targetHeightPx = `${targetHeight}px`;

      for (const element of rowElements) {
        if (element.style.height !== targetHeightPx) {
          element.style.height = targetHeightPx;
        }
      }
    }
  }
</script>

<SectionPanel title={copy.system.title} description={copy.system.description}>
  <div class="grid gap-2.5 md:grid-cols-2 xl:grid-cols-5">
    {#each cards as card, index (card.label)}
      {@const Icon = card.icon}

      <article
        class="flex h-full flex-col rounded-[8px] border border-shell-200/85 bg-white px-3 py-2.5 shadow-[0_8px_18px_rgba(15,23,42,0.04)]"
      >
        <header bind:this={headerElements[index]} class="flex flex-col">
          <div class="flex items-center gap-2.5">
            <span
              class={`flex h-9 w-9 shrink-0 items-center justify-center rounded-[8px] ${card.iconClass}`}
            >
              <Icon size={18} strokeWidth={1.9} />
            </span>
            <span class="text-[13.5px] font-medium leading-5 text-shell-600">
              {card.label}
            </span>
          </div>

          <div
            class="mt-2.5 flex min-h-0 flex-1 items-start"
            title={card.value}
          >
            <strong
              class="card-primary-value block text-[1rem] font-semibold leading-[1.35] text-shell-950"
            >
              {card.value}
            </strong>
          </div>
        </header>

        <div class="mt-2.5 space-y-1.5 border-t border-shell-150 pt-2.5">
          {#each card.metrics as metric (metric.label)}
            <div
              class="grid grid-cols-[5.5rem_minmax(0,1fr)] items-start gap-x-2.5 text-[12.5px] leading-5"
            >
              <span class="break-words text-shell-500">{metric.label}</span>
              <span
                class="card-metric-value break-words text-right font-medium text-shell-800"
                title={metric.value}
              >
                {metric.value}
              </span>
            </div>
          {/each}
        </div>
      </article>
    {/each}
  </div>
</SectionPanel>

<style>
  .card-primary-value,
  .card-metric-value {
    display: -webkit-box;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    overflow: hidden;
  }
</style>
