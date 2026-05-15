<script lang="ts">
  import Info from "@lucide/svelte/icons/info";
  import { i18nState } from "$lib/i18n";
  import RefreshButton from "$lib/ui/RefreshButton.svelte";
  import type { AcquireMode } from "../view-model/acquire-rules";
  import type { AcquireWorkspaceViewModel } from "../view-model/workspace.svelte";
  import ImportFormPanel from "./ImportFormPanel.svelte";
  import InstallFormPanel from "./InstallFormPanel.svelte";
  import OnlineDistroList from "./OnlineDistroList.svelte";

  type Props = { model: AcquireWorkspaceViewModel; activeMode: AcquireMode };
  let props: Props = $props();
  const model = $derived(props.model);
  const copy = $derived($i18nState.copy.acquire);
  const commonCopy = $derived($i18nState.copy.common);
  const cardClass = "panel-surface flex h-full min-h-[400px] min-w-0 flex-col p-3";
</script>

{#if props.activeMode === "store"}
  <div class="grid auto-rows-fr grid-cols-[minmax(0,1.25fr)_minmax(340px,0.75fr)] items-stretch gap-[11px]">
    <article class={cardClass}>
      <div class="flex items-center justify-between gap-3">
        <div class="flex min-w-0 items-baseline gap-2"><h3 class="text-[1.04rem] font-semibold leading-tight text-shell-950">{copy.onlineList.title}</h3><span class="text-[12px] leading-4 text-shell-500">{copy.onlineList.count(model.onlineDistros.length)}</span></div>
        <RefreshButton label={commonCopy.refreshList} refreshing={model.refreshing} disabled={model.refreshDisabled} color="secondary" size="sm" className="!min-h-0 !gap-1.5 !px-3 !py-[5px] !text-[13px]" onclick={() => void model.callbacks.refreshOnlineDistros()} />
      </div>
      <div class="mt-2.5 flex min-h-0 flex-1 flex-col"><OnlineDistroList {model} /></div>
    </article>
    <article class={cardClass}>
      <h3 class="text-[1.04rem] font-semibold text-shell-950">{copy.sections.installConfig}</h3>
      <div class="mt-2.5 flex min-h-0 flex-1 flex-col"><InstallFormPanel {model} /></div>
    </article>
  </div>
{:else}
  <div class="grid grid-cols-1 items-stretch gap-[11px] xl:grid-cols-[minmax(0,760px)_minmax(300px,1fr)]">
    <article class={cardClass}>
      <h3 class="text-[1.04rem] font-semibold text-shell-950">{copy.sections.importConfig}</h3>
      <div class="mt-2.5 flex min-h-0 flex-1 flex-col"><ImportFormPanel {model} /></div>
    </article>
    <aside class={cardClass}>
      <div class="flex items-center justify-between gap-3"><h3 class="text-[1.04rem] font-semibold text-shell-950">{copy.importGuide.title}</h3><Info size={17} strokeWidth={1.9} class="shrink-0 text-shell-600" /></div>
      <div class="mt-3 grid gap-3">
        {#each copy.importGuide.items as item}<section class="rounded-[10px] border-[0.5px] border-shell-200/80 bg-white px-3.5 py-3"><strong class="block text-[15px] font-semibold text-shell-950">{item.title}</strong><p class="mt-1 text-[13px] leading-5 text-shell-600">{item.description}</p></section>{/each}
      </div>
    </aside>
  </div>
{/if}
