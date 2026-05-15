<script lang="ts">
  import Square from "@lucide/svelte/icons/square";

  import { i18nState } from "$lib/i18n";
  import PageHeader from "$lib/ui/PageHeader.svelte";
  import RefreshButton from "$lib/ui/RefreshButton.svelte";
  import StatusNotice from "$lib/ui/StatusNotice.svelte";

  import DistroWorkspaceSection from "./components/DistroWorkspaceSection.svelte";
  import { createDistroWorkspaceViewModel } from "./view-model/distro-workspace-view-model.svelte";

  const viewModel = createDistroWorkspaceViewModel();
  const distrosCopy = $derived($i18nState.copy.distros);
</script>

<div class="page-stack">
  <PageHeader
    eyebrow={distrosCopy.page.eyebrow}
    title={distrosCopy.page.title}
    description={distrosCopy.page.description}
  >
    {#snippet actions()}
      <RefreshButton
        label={viewModel.view.refreshButton.label}
        refreshing={viewModel.view.refreshButton.refreshing}
        refreshingLabel={viewModel.view.refreshButton.refreshingLabel}
        disabled={viewModel.view.refreshButton.disabled}
        onclick={() => void viewModel.callbacks.refresh()}
      />
      <RefreshButton
        label={viewModel.view.shutdownButton.label}
        icon={Square}
        color="danger"
        refreshing={viewModel.view.shutdownButton.running}
        refreshingLabel={viewModel.view.shutdownButton.refreshingLabel}
        disabled={viewModel.view.shutdownButton.disabled}
        onclick={() => void viewModel.callbacks.shutdownAll()}
      />
    {/snippet}
  </PageHeader>

  {#each viewModel.view.notices as notice (notice.key)}
    <StatusNotice
      tone={notice.tone}
      title={notice.title}
      message={notice.message}
    />
  {/each}

  <DistroWorkspaceSection
    section={viewModel.view.section}
    callbacks={viewModel.callbacks}
  />
</div>
