<script lang="ts">
  import EmptyState from "$lib/ui/EmptyState.svelte";
  import { i18nState } from "$lib/i18n";
  import SectionPanel from "$lib/ui/SectionPanel.svelte";

  import type {
    DistroWorkspaceCallbacks,
    DistroWorkspaceSectionView,
  } from "../view-model/distro-workspace-types";
  import DistroRow from "./DistroRow.svelte";

  type Props = {
    section: DistroWorkspaceSectionView;
    callbacks: DistroWorkspaceCallbacks;
  };

  let { section, callbacks }: Props = $props();

  const distroCountClass = "flex items-center justify-end gap-2 text-right";
  const distroCountLabelClass = "text-[11px] font-medium text-shell-500";
  const distroCountValueClass =
    "inline-flex min-w-8 items-center justify-center rounded-[8px] bg-shell-950 px-2.5 py-1 text-[16px] font-semibold leading-none tracking-[-0.03em] text-white";
</script>

<SectionPanel title={section.title}>
  {#snippet meta()}
    <span class={distroCountClass}>
      <span class={distroCountValueClass}>
        {section.count}
      </span>
      <span class={distroCountLabelClass}>
        {$i18nState.copy.distros.section.countLabel}
      </span>
    </span>
  {/snippet}

  {#if section.state !== "ready" && section.emptyTitle && section.emptyMessage}
    <EmptyState title={section.emptyTitle} message={section.emptyMessage} />
  {:else}
    <ul class="grid gap-2.5">
      {#each section.rows as row (row.name)}
        <DistroRow {row} {callbacks} />
      {/each}
    </ul>
  {/if}
</SectionPanel>
