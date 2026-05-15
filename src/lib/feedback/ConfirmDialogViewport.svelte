<script lang="ts">
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";

  import { i18nState } from "$lib/i18n";

  import {
    confirmDialogState,
    resolveConfirmDialog,
  } from "./confirm-dialog";

  const toneClassMap = {
    warning: {
      iconWrap: "bg-amber-50 text-amber-600",
      title: "text-amber-800",
      accent:
        "border-amber-200/80 bg-[linear-gradient(180deg,rgba(255,251,235,0.98),rgba(255,255,255,0.99))]",
    },
    danger: {
      iconWrap: "bg-rose-50 text-rose-600",
      title: "text-rose-800",
      accent:
        "border-rose-200/80 bg-[linear-gradient(180deg,rgba(255,241,242,0.98),rgba(255,255,255,0.99))]",
    },
    info: {
      iconWrap: "bg-accent-50 text-accent-600",
      title: "text-accent-800",
      accent:
        "border-accent-200/80 bg-[linear-gradient(180deg,rgba(238,247,255,0.98),rgba(255,255,255,0.99))]",
    },
  } as const;

  const buttonVariantClassMap = {
    primary:
      "border-accent-600 bg-accent-600 text-white hover:bg-accent-700 hover:border-accent-700 focus-visible:ring-accent-300",
    danger:
      "border-rose-600 bg-rose-600 text-white hover:bg-rose-700 hover:border-rose-700 focus-visible:ring-rose-300",
    secondary:
      "border-shell-200 bg-white text-shell-700 hover:bg-shell-50 hover:border-shell-300 focus-visible:ring-shell-300",
  } as const;

  let activeElementBeforeOpen = $state<HTMLElement | null>(null);

  function handleKeydown(event: KeyboardEvent): void {
    if (!$confirmDialogState) {
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      resolveConfirmDialog("cancel");
    }
  }

  $effect(() => {
    const dialog = $confirmDialogState;

    if (
      typeof document !== "undefined" &&
      dialog &&
      activeElementBeforeOpen === null
    ) {
      activeElementBeforeOpen =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
    }

    if (!dialog && activeElementBeforeOpen) {
      activeElementBeforeOpen.focus();
      activeElementBeforeOpen = null;
    }
  });

  $effect(() => {
    return () => {
      activeElementBeforeOpen = null;
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $confirmDialogState}
  {@const toneClasses = toneClassMap[$confirmDialogState.tone]}

  <div class="fixed inset-0 z-[60] flex items-center justify-center p-4">
    <button
      aria-label={$i18nState.copy.common.feedback.dismissConfirmDialog}
      class="absolute inset-0 border-0 bg-shell-950/40 backdrop-blur-[2px]"
      onclick={() => resolveConfirmDialog("cancel")}
      type="button"
    ></button>

    <div
      aria-labelledby="confirm-dialog-title"
      aria-modal="true"
      class={`relative z-10 w-full max-w-[28rem] rounded-[16px] border-[0.5px] p-4 shadow-[0_24px_60px_rgba(17,26,39,0.24)] backdrop-blur-xl ${toneClasses.accent}`}
      role="dialog"
    >
      <div class="flex items-start gap-3">
        <span
          class={`flex h-11 w-11 shrink-0 items-center justify-center rounded-[12px] ${toneClasses.iconWrap}`}
        >
          <TriangleAlert size={20} strokeWidth={2.1} />
        </span>

        <div class="min-w-0 flex-1">
          <h2
            class={`text-[1rem] font-semibold tracking-[-0.025em] ${toneClasses.title}`}
            id="confirm-dialog-title"
          >
            {$confirmDialogState.title}
          </h2>
          <p class="mt-2 text-[13px] leading-6 text-shell-600">
            {$confirmDialogState.message}
          </p>
        </div>
      </div>

      <div class="mt-4 flex flex-wrap justify-end gap-2">
        {#each $confirmDialogState.actions as action}
          <button
            class={`inline-flex min-h-9 items-center justify-center rounded-[10px] border px-4 py-2 text-[13px] font-semibold transition duration-150 focus-visible:outline-none focus-visible:ring-2 ${buttonVariantClassMap[action.variant]}`}
            onclick={() => resolveConfirmDialog(action.id)}
            type="button"
          >
            {action.label}
          </button>
        {/each}
      </div>
    </div>
  </div>
{/if}
