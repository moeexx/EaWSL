<script lang="ts">
  import X from "@lucide/svelte/icons/x";
  import { fly } from "svelte/transition";

  import { i18nState } from "$lib/i18n";

  import { dismissToast, toastState } from "./toasts";

  const toneClassMap = {
    error: {
      container:
        "border-red-200 bg-[linear-gradient(135deg,rgba(254,242,242,0.96),rgba(255,255,255,0.98))]",
      dot: "bg-red-500",
      title: "text-red-700",
      button:
        "text-red-500 hover:bg-red-50 hover:text-red-700 focus-visible:ring-red-200",
    },
    info: {
      container:
        "border-accent-200 bg-[linear-gradient(135deg,rgba(238,247,255,0.96),rgba(255,255,255,0.98))]",
      dot: "bg-accent-500",
      title: "text-accent-700",
      button:
        "text-accent-500 hover:bg-accent-50 hover:text-accent-700 focus-visible:ring-accent-200",
    },
    success: {
      container:
        "border-emerald-200 bg-[linear-gradient(135deg,rgba(236,253,245,0.96),rgba(255,255,255,0.98))]",
      dot: "bg-emerald-500",
      title: "text-emerald-700",
      button:
        "text-emerald-500 hover:bg-emerald-50 hover:text-emerald-700 focus-visible:ring-emerald-200",
    },
    warning: {
      container:
        "border-amber-200 bg-[linear-gradient(135deg,rgba(255,251,235,0.96),rgba(255,255,255,0.98))]",
      dot: "bg-amber-500",
      title: "text-amber-700",
      button:
        "text-amber-500 hover:bg-amber-50 hover:text-amber-700 focus-visible:ring-amber-200",
    },
  } as const;
</script>

<div
  class="pointer-events-none fixed right-3 top-12 z-50 flex w-[min(26rem,calc(100vw-1.5rem))] flex-col gap-2 sm:right-4 sm:top-14 sm:w-[24rem]"
>
  {#each $toastState as toast (toast.id)}
    {@const toneClasses = toneClassMap[toast.tone]}

    <section
      class={`pointer-events-auto rounded-[10px] border-[0.5px] px-3 py-3 shadow-[0_12px_32px_rgba(17,26,39,0.12)] backdrop-blur-xl ${toneClasses.container}`}
      in:fly={{ y: -8, duration: 160 }}
      out:fly={{ y: -8, duration: 140 }}
    >
      <div class="flex items-start gap-3">
        <span
          aria-hidden="true"
          class={`mt-1.5 h-2.5 w-2.5 shrink-0 rounded-full ${toneClasses.dot}`}
        ></span>

        <div class="min-w-0 flex-1">
          <p class={`text-sm font-semibold ${toneClasses.title}`}>{toast.title}</p>
          <p class="mt-1 text-[13px] leading-5 text-shell-600">
            {toast.message}
          </p>
        </div>

        <button
          aria-label={$i18nState.copy.common.feedback.dismissToast}
          class={`flex h-7 w-7 shrink-0 items-center justify-center rounded-[8px] transition duration-150 focus-visible:outline-none focus-visible:ring-2 ${toneClasses.button}`}
          onclick={() => dismissToast(toast.id)}
          type="button"
        >
          <X size={15} strokeWidth={2.2} />
        </button>
      </div>
    </section>
  {/each}
</div>
