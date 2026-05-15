<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Copy from "@lucide/svelte/icons/copy";
  import Info from "@lucide/svelte/icons/info";

  import Button from "$lib/ui/Button.svelte";
  import SectionPanel from "$lib/ui/SectionPanel.svelte";
  import type { AppCopy } from "$lib/i18n";

  import type { OverviewInfoItem } from "../view-model/overview-view-model";

  type Props = {
    items: OverviewInfoItem[];
    copy: AppCopy["overview"];
  };

  let { items, copy }: Props = $props();
  let copied = $state(false);
  let copyFeedbackTimer: ReturnType<typeof setTimeout> | null = null;
  const copyButtonIcon = $derived(copied ? Check : Copy);
  const copyButtonLabel = $derived(copied ? copy.wsl.copied : copy.wsl.copy);
  let copyText = $derived(
    items.map((item) => `${item.label}: ${item.value}`).join("\n"),
  );

  function clearCopyFeedbackTimer(): void {
    if (!copyFeedbackTimer) {
      return;
    }

    clearTimeout(copyFeedbackTimer);
    copyFeedbackTimer = null;
  }

  function showCopyFeedback(): void {
    copied = true;
    clearCopyFeedbackTimer();

    copyFeedbackTimer = setTimeout(() => {
      copied = false;
      copyFeedbackTimer = null;
    }, 1800);
  }

  async function copyWslInfo(): Promise<void> {
    if (!navigator.clipboard || copyText.trim() === "") {
      return;
    }

    try {
      await navigator.clipboard.writeText(copyText);
      showCopyFeedback();
    } catch {
      copied = false;
    }
  }

  $effect(() => {
    return () => clearCopyFeedbackTimer();
  });
</script>

<SectionPanel
  title={copy.wsl.title}
  description={copy.wsl.description}
>
  {#snippet icon()}
  <span
    class="flex h-10 w-10 items-center justify-center rounded-full bg-shell-100 text-shell-800"
  >
    <Info size={20} strokeWidth={2} />
  </span>
  {/snippet}

  {#snippet actions()}
  <Button
    label={copyButtonLabel}
    icon={copyButtonIcon}
    variant="secondary"
    className="gap-2 rounded-[10px] border border-shell-200 bg-white px-4 py-2 text-[14px] font-medium text-shell-800 hover:border-shell-300 hover:bg-shell-50"
    onclick={() => void copyWslInfo()}
  />
  {/snippet}

  <div class="border-t border-shell-200/80 pt-3">
    <div class="grid gap-0 divide-y divide-shell-150/90">
      {#each items as item (item.label)}
        <div
          class="grid gap-1 py-2.5 sm:grid-cols-[160px_minmax(0,1fr)] sm:items-start sm:gap-4"
        >
          <span class="text-[14px] font-semibold text-shell-950">
            {item.label}
          </span>
          <span class="min-w-0 break-all text-[14px] leading-6 text-shell-500">
            {item.value}
          </span>
        </div>
      {/each}
    </div>
  </div>
</SectionPanel>
