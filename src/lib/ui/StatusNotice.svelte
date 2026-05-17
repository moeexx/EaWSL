<script lang="ts">
  type NoticeTone = "error" | "info" | "success" | "warning";

  type Props = {
    tone?: NoticeTone;
    title: string;
    message: string;
  };

  let { tone = "info", title, message }: Props = $props();

  const toneClassMap = {
    error: {
      container:
        "border-red-200 bg-[linear-gradient(135deg,rgba(254,242,242,0.95),rgba(255,255,255,0.95))]",
      dot: "bg-red-500",
      title: "text-red-700",
    },
    info: {
      container:
        "border-accent-200 bg-[linear-gradient(135deg,rgba(238,247,255,0.95),rgba(255,255,255,0.95))]",
      dot: "bg-accent-500",
      title: "text-accent-700",
    },
    success: {
      container:
        "border-emerald-200 bg-[linear-gradient(135deg,rgba(236,253,245,0.95),rgba(255,255,255,0.95))]",
      dot: "bg-emerald-500",
      title: "text-emerald-700",
    },
    warning: {
      container:
        "border-amber-200 bg-[linear-gradient(135deg,rgba(255,251,235,0.95),rgba(255,255,255,0.95))]",
      dot: "bg-amber-500",
      title: "text-amber-700",
    },
  } as const;

  const toneClasses = $derived(toneClassMap[tone]);
</script>

<section
  class={`rounded-[8px] border-[0.5px] px-3 py-3 backdrop-blur-sm ${toneClasses.container}`}
>
  <div class="flex items-start gap-3">
    <span
      aria-hidden="true"
      class={`mt-2 h-2.5 w-2.5 shrink-0 rounded-full ${toneClasses.dot}`}
    ></span>

    <div class="min-w-0">
      <p class={`text-sm font-semibold ${toneClasses.title}`}>{title}</p>
      <p class="mt-1 text-[13px] leading-5 text-shell-600">{message}</p>
    </div>
  </div>
</section>
