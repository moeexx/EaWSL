<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    title: string;
    description?: string;
    icon?: Snippet;
    actions?: Snippet;
    meta?: Snippet;
    children?: Snippet;
  };

  let {
    title,
    description = undefined,
    icon,
    actions,
    meta,
    children,
  }: Props = $props();

  const hasHeading = $derived(title.trim().length > 0 || Boolean(description));
  const hasHeader = $derived(
    hasHeading || Boolean(icon) || Boolean(actions) || Boolean(meta),
  );
</script>

<section class="panel-surface p-[14px]">
  <div class="flex flex-col gap-3">
    {#if hasHeader}
      <div class="flex items-start justify-between gap-3">
        <div class="flex min-w-0 max-w-3xl items-start gap-3">
          {#if icon}
            <div class="shrink-0">
              {@render icon()}
            </div>
          {/if}

          {#if hasHeading}
            <div class="min-w-0">
              {#if title.trim().length > 0}
                <h2
                  class="text-[1.2rem] font-semibold tracking-[-0.025em] text-shell-950"
                >
                  {title}
                </h2>
              {/if}

              {#if description}
                <p class="mt-2 text-[13px] leading-5 text-shell-600">
                  {description}
                </p>
              {/if}
            </div>
          {/if}
        </div>

        {#if actions || meta}
          <div class="shrink-0 self-start">
            {#if actions}
              {@render actions()}
            {:else}
              {@render meta?.()}
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    {@render children?.()}
  </div>
</section>
