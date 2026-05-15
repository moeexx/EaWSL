<script lang="ts">
  import type { SpaceTone } from "../view-model/acquire-rules";
  type Props = { tone: SpaceTone; message: string };
  let { tone, message }: Props = $props();
  const toneClass = $derived(tone === "success" ? "border-emerald-200/80 bg-emerald-50/80 text-emerald-700" : tone === "error" ? "border-rose-200/80 bg-rose-50/80 text-rose-700" : tone === "info" ? "border-accent-200/80 bg-accent-50/80 text-accent-700" : "border-shell-200/80 bg-shell-50/80 text-shell-600");
</script>

<div class={`flex min-h-[56px] items-center rounded-[10px] border-[0.5px] px-3 py-2 text-[12px] leading-5 ${toneClass}`}>{message}</div>
