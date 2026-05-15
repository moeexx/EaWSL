<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    message: string;
    className?: string;
    children?: Snippet;
  };

  let { title, message, className = "", children }: Props = $props();
</script>

<div class={`ui-empty-state ${className}`}>
  <strong class="text-[15px] font-semibold text-shell-950">
    {title}
  </strong>
  <p class="mt-2 text-[13px] leading-6 text-shell-600">
    {message}
  </p>
  {@render children?.()}
</div>
