<script lang="ts">
  import SectionPanel from "$lib/ui/SectionPanel.svelte";
  import type { AppCopy } from "$lib/i18n";

  import type { OverviewSystemInfoCard } from "../view-model/system-overview-cards";

  type Props = {
    cards: OverviewSystemInfoCard[];
    copy: AppCopy["overview"];
  };

  let { cards, copy }: Props = $props();
</script>

<SectionPanel
  title={copy.system.title}
  description={copy.system.description}
>
  <div class="grid gap-2.5 md:grid-cols-2 xl:grid-cols-5">
    {#each cards as card (card.label)}
      {@const Icon = card.icon}

      <article
        class="flex flex-col rounded-[14px] border border-shell-200/85 bg-white px-3.5 py-3 shadow-[0_8px_18px_rgba(15,23,42,0.04)]"
      >
        <div class="flex items-center gap-2.5">
          <span
            class={`flex h-9 w-9 items-center justify-center rounded-[10px] ${card.iconClass}`}
          >
            <Icon size={18} strokeWidth={1.9} />
          </span>
          <span class="text-[14px] font-medium text-shell-600">
            {card.label}
          </span>
        </div>

        <div class="mt-3">
          <strong
            class="block text-[1rem] font-semibold leading-[1.35] text-shell-950"
          >
            {card.value}
          </strong>
        </div>

        <div class="mt-3 space-y-1.5 border-t border-shell-150 pt-3">
          {#each card.metrics as metric (metric.label)}
            <div
              class="flex items-start justify-between gap-3 text-[12.5px] leading-5"
            >
              <span class="text-shell-500">{metric.label}</span>
              <span class="text-right font-medium text-shell-800">
                {metric.value}
              </span>
            </div>
          {/each}
        </div>
      </article>
    {/each}
  </div>
</SectionPanel>
