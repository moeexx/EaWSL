<script lang="ts">
  import Download from "@lucide/svelte/icons/download";
  import ShoppingBag from "@lucide/svelte/icons/shopping-bag";

  import { i18nState } from "$lib/i18n";
  import Button from "$lib/ui/Button.svelte";
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
      ? "!h-10 !w-10 !rounded-[10px] !border-accent-200 !bg-accent-100 !p-0 !text-accent-800"
      : "!h-10 !w-10 !rounded-[10px] !border-shell-200/80 !bg-white !p-0 !text-shell-600 hover:!bg-shell-50";
  }
</script>

<div class="page-stack">
  <PageHeader
    eyebrow={acquireCopy.page.eyebrow}
    title={acquireCopy.page.title}
    description={acquireCopy.page.description}
  >
    {#snippet actions()}
      <div class="flex items-center gap-2">
        {#each acquireModeOrder as mode}
          {@const ModeIcon = modeIconMap[mode]}
          <Button
            ariaLabel={acquireCopy.modes[mode]}
            ariaPressed={activeMode === mode}
            icon={ModeIcon}
            variant="secondary"
            className={getModeClass(mode)}
            onclick={() => {
              activeMode = mode;
            }}
            title={acquireCopy.modes[mode]}
          />
        {/each}
      </div>
    {/snippet}
  </PageHeader>

  <AcquireWorkspaceSection {model} {activeMode} />
</div>
