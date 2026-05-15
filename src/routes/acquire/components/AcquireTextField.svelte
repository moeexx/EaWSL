<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    id: string; label: string; value: string; placeholder: string; invalid: boolean; error: string | null;
    required?: boolean; inputRequired?: boolean; disabled?: boolean; oninput: (value: string) => void; actions?: Snippet;
  };

  let { id, label, value, placeholder, invalid, error, required = false, inputRequired = false, disabled = false, oninput, actions }: Props = $props();
  const inputClass = $derived(`min-h-[36px] rounded-[10px] border-[0.5px] bg-white px-3 text-[14px] text-shell-900 transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-200 ${invalid ? "border-rose-200" : "border-shell-200/80"} ${disabled ? "cursor-not-allowed opacity-60" : ""}`);
</script>

<div class="grid gap-1.5">
  <div class="flex items-start justify-between gap-3">
    <label class="text-[14px] font-medium text-shell-600" for={id}>{label}{#if required}<span class="text-rose-600">*</span>{/if}</label>
    {#if error}<p class="min-w-0 text-right text-[12px] leading-5 text-rose-700">{error}</p>{/if}
  </div>

  {#if actions}
    <div class="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto]">
      <input {id} aria-invalid={invalid} class={inputClass} {disabled} oninput={(event) => oninput(event.currentTarget.value)} {placeholder} required={inputRequired} type="text" {value} />
      {@render actions()}
    </div>
  {:else}
    <input {id} aria-invalid={invalid} class={inputClass} {disabled} oninput={(event) => oninput(event.currentTarget.value)} {placeholder} required={inputRequired} type="text" {value} />
  {/if}
</div>
