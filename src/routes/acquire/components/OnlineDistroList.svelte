<script lang="ts">
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import { i18nState } from "$lib/i18n";
  import Button from "$lib/ui/Button.svelte";
  import type { AcquireWorkspaceViewModel } from "../view-model/workspace.svelte";

  type Props = { model: AcquireWorkspaceViewModel };
  let { model }: Props = $props();
  const copy = $derived($i18nState.copy.acquire.onlineList);
  const commonCopy = $derived($i18nState.copy.common);
  const hasDistros = $derived(model.onlineDistros.length > 0);
  const showLoading = $derived(model.refreshing || model.queryState.onlineDistros.activity === "loading" || model.queryState.onlineDistros.activity === "refreshing");
  const cardClass = (selected: boolean) => `group relative flex max-h-[74px] justify-start gap-2.5 overflow-hidden rounded-[10px] border-[0.5px] py-2.5 text-left ${selected ? "!border-accent-200 !bg-accent-50 pl-3 pr-4" : "!border-shell-200/80 !bg-white px-3 hover:!bg-shell-50"}`;
  const loading = (overlay: boolean) => overlay ? "absolute inset-0 z-10 flex items-center justify-center rounded-[8px] bg-white/55 text-shell-700 backdrop-blur-[1px]" : "flex flex-1 items-center justify-center text-shell-600";
  const stateCard = "flex flex-1 items-center justify-center rounded-[10px] border-[0.5px] px-4 py-8 text-center";
</script>

<div class="flex h-full min-h-[320px] min-w-0 flex-col gap-3">
  {#if hasDistros}
    <div class="relative min-h-0 flex-1">
      <div class={`grid grid-cols-2 auto-rows-max content-start items-start gap-2 min-[1100px]:grid-cols-3 ${showLoading ? "opacity-45" : ""}`}>
        {#each model.onlineDistros as distro (distro.name)}
          {@const selected = model.selectedDistroName === distro.name}
          <Button ariaPressed={selected} variant="secondary" className={cardClass(selected)} onclick={() => model.callbacks.selectDistro(distro.name)}>
            <img alt="" class="h-9 w-9 shrink-0 rounded-[9px] border border-shell-200 bg-shell-50 object-contain p-1.5" src={distro.logoSrc} />
            <div class="min-w-0 flex-1"><strong class="block truncate text-[14px] font-semibold text-shell-950">{distro.friendly_name}</strong><p class="mt-0.5 truncate text-[12px] leading-4 text-shell-500">{distro.name}</p></div>
            {#if selected}<span aria-hidden="true" class="absolute bottom-2 right-0 top-2 w-1 rounded-l-full bg-accent-700"></span>{/if}
          </Button>
        {/each}
      </div>
      {#if showLoading}<div class={loading(true)} role="status"><div class="inline-flex items-center gap-2.5 text-[15px] font-semibold"><LoaderCircle size={21} strokeWidth={2} class="animate-spin text-accent-700" /><span>{commonCopy.loading}</span></div></div>{/if}
    </div>
  {:else if showLoading || model.onlineState === "loading"}
    <div class={loading(false)} role="status"><div class="inline-flex items-center gap-2.5 text-[15px] font-semibold"><LoaderCircle size={21} strokeWidth={2} class="animate-spin text-accent-700" /><span>{commonCopy.loading}</span></div></div>
  {:else if model.onlineState === "error" || model.onlineState === "recovering"}
    <div class={`${stateCard} border-rose-200/80 bg-rose-50/80 text-rose-700`}><div><p class="text-[14px] font-semibold">{commonCopy.readFailed}</p><p class="mt-1 text-[12px] leading-5">{copy.readFailedMessage}</p></div></div>
  {:else}
    <div class={`${stateCard} border-shell-200/80 bg-shell-50/80 text-shell-600`}><div><p class="text-[14px] font-semibold text-shell-700">{copy.emptyTitle}</p><p class="mt-1 text-[12px] leading-5">{copy.emptyMessage}</p></div></div>
  {/if}
</div>
