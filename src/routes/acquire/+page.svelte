<script lang="ts">
  import Download from "@lucide/svelte/icons/download";
  import ShoppingBag from "@lucide/svelte/icons/shopping-bag";

  import { i18nState } from "$lib/i18n";
  import PageHeader from "$lib/ui/PageHeader.svelte";

  import AcquireWorkspaceSection from "./components/AcquireWorkspaceSection.svelte";
  import {
    acquireModeOrder,
    type AcquireMode,
  } from "./view-model/acquire-rules";
  import { createAcquireWorkspaceViewModel } from "./view-model/workspace.svelte";

  const model = createAcquireWorkspaceViewModel();
  let activeMode = $state<AcquireMode>("store");
  const acquireCopy = $derived($i18nState.copy.acquire);

  const modeIconMap = {
    store: ShoppingBag,
    import: Download,
  } satisfies Record<AcquireMode, typeof ShoppingBag>;

  function getModeClass(mode: AcquireMode): string {
    return activeMode === mode
      ? "border-shell-200 bg-white text-accent-800 shadow-[0_3px_8px_rgba(32,123,229,0.12)]"
      : "border-transparent text-shell-500 hover:bg-white/65 hover:text-shell-800";
  }
</script>

<div class="page-stack">
  <PageHeader
    eyebrow={acquireCopy.page.eyebrow}
    title={acquireCopy.page.title}
    description={acquireCopy.page.description}
  >
    {#snippet actions()}
      <div
        aria-label={acquireCopy.page.title}
        class="inline-flex items-center gap-1 rounded-[8px] border-[0.5px] border-shell-200/80 bg-shell-100/85 p-1"
        role="tablist"
      >
        {#each acquireModeOrder as mode}
          {@const ModeIcon = modeIconMap[mode]}
          <button
            aria-selected={activeMode === mode}
            class={`inline-flex h-8 min-w-[86px] items-center justify-center gap-2 rounded-[8px] border-[0.5px] px-3 text-[13px] font-semibold leading-none transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-200 ${getModeClass(mode)}`}
            onclick={() => {
              activeMode = mode;
            }}
            role="tab"
            title={acquireCopy.modes[mode]}
            type="button"
          >
            <ModeIcon
              class="shrink-0"
              size={16}
              style="width: 16px; height: 16px;"
              strokeWidth={2}
            />
            <span>{acquireCopy.modes[mode]}</span>
          </button>
        {/each}
      </div>
    {/snippet}
  </PageHeader>

  <AcquireWorkspaceSection {model} {activeMode} />
</div>
