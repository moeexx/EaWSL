<script lang="ts">
  import { untrack } from "svelte";

  import PageHeader from "$lib/ui/PageHeader.svelte";
  import StatusNotice from "$lib/ui/StatusNotice.svelte";
  import { i18nState } from "$lib/i18n";
  import { runLoggedRefreshFlow } from "$lib/shared/frontend-logs";
  import { queryCache, refreshQueries } from "$lib/query-cache";
  import { pushToast } from "$lib/feedback/toasts";

  import RefreshButton from "$lib/ui/RefreshButton.svelte";
  import SystemOverviewSection from "./components/SystemOverviewSection.svelte";
  import WslOverviewSection from "./components/WslOverviewSection.svelte";
  import { buildOverviewViewModel } from "./view-model/overview-view-model";

  const overviewCopy = $derived($i18nState.copy.overview);
  let overviewViewModel = $derived(
    buildOverviewViewModel($queryCache, $i18nState.copy),
  );
  let overviewRefreshing = $state(false);

  $effect(() => {
    const logSubject = untrack(() => overviewCopy.page.logSubject);
    void runLoggedRefreshFlow(logSubject, "page-enter", () =>
      refreshOverviewPlan("page-enter"),
    );
  });

  async function handleRefreshOverview(): Promise<void> {
    overviewRefreshing = true;

    try {
      const result = await runLoggedRefreshFlow(
        overviewCopy.page.logSubject,
        "manual",
        () => refreshOverviewPlan("manual"),
      );

      if (hasFailedOverviewResult(result)) {
        pushToast({
          tone: "error",
          title: overviewCopy.refresh.failedTitle,
          message: overviewCopy.refresh.failedMessage,
        });
        return;
      }

      pushToast({
        tone: hasRecoveringOverviewResult(result) ? "warning" : "success",
        title: overviewCopy.refresh.completedTitle,
        message: hasRecoveringOverviewResult(result)
          ? overviewCopy.refresh.recoveringMessage
          : overviewCopy.refresh.successMessage,
      });
    } finally {
      overviewRefreshing = false;
    }
  }

  async function refreshOverviewPlan(reason: "page-enter" | "manual") {
    return refreshQueries({
      foreground: [
        { key: "systemOverview", scope: "full" },
        "wslVersion",
        "distros",
      ],
      background: [],
      reason,
      foregroundMinDurationMs: reason === "manual" ? 500 : 0,
    });
  }

  function hasFailedOverviewResult(
    result: Awaited<ReturnType<typeof refreshOverviewPlan>>,
  ): boolean {
    return (
      result.systemOverview?.kind === "failed" ||
      result.wslVersion?.kind === "failed" ||
      result.distros?.kind === "failed"
    );
  }

  function hasRecoveringOverviewResult(
    result: Awaited<ReturnType<typeof refreshOverviewPlan>>,
  ): boolean {
    return (
      result.systemOverview?.kind === "recovering" ||
      result.wslVersion?.kind === "recovering" ||
      result.distros?.kind === "recovering"
    );
  }
</script>

<div class="page-stack">
  <PageHeader
    eyebrow={overviewCopy.page.eyebrow}
    title={overviewCopy.page.title}
    description={overviewCopy.page.description}
  >
    {#snippet actions()}
    <RefreshButton
      label={overviewCopy.page.refreshLabel}
      refreshing={overviewRefreshing}
      onclick={() => void handleRefreshOverview()}
    />
    {/snippet}
  </PageHeader>

  {#each overviewViewModel.notices as notice (notice.key)}
    <StatusNotice
      tone={notice.tone}
      title={notice.title}
      message={notice.message}
    />
  {/each}

  <SystemOverviewSection
    cards={overviewViewModel.systemCards}
    copy={overviewCopy}
  />
  <WslOverviewSection
    items={overviewViewModel.wslInfoItems}
    copy={overviewCopy}
  />
</div>
