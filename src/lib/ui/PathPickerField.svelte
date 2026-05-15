<script lang="ts">
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import type { Snippet } from "svelte";

  import Button, { type ButtonVariant } from "./Button.svelte";

  type IconComponent = typeof FolderOpen;

  type Props = {
    id: string;
    label: string;
    value: string;
    error: string | null;
    placeholder: string;
    chooseLabel: string;
    chooseIcon?: IconComponent | null;
    required?: boolean;
    inputRequired?: boolean;
    disabled?: boolean;
    chooseDisabled?: boolean;
    buttonVariant?: ButtonVariant;
    buttonClassName?: string;
    inputClassName?: string;
    labelClassName?: string;
    oninput: (value: string) => void;
    onchoose: () => void;
    after?: Snippet;
  };

  let {
    id,
    label,
    value,
    error,
    placeholder,
    chooseLabel,
    chooseIcon = FolderOpen,
    required = false,
    inputRequired = false,
    disabled = false,
    chooseDisabled = false,
    buttonVariant = "secondary",
    buttonClassName = "min-h-[36px]",
    inputClassName = "",
    labelClassName = "text-[14px] font-medium text-shell-600",
    oninput,
    onchoose,
    after,
  }: Props = $props();

  const invalid = $derived(error !== null);
  const errorId = $derived(invalid ? `${id}-error` : undefined);
  const inputClass = $derived(
    `min-h-[36px] rounded-[10px] border-[0.5px] bg-white px-3 text-[14px] text-shell-900 transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-200 disabled:cursor-not-allowed disabled:bg-shell-50 disabled:text-shell-500 ${
      invalid ? "border-rose-200" : "border-shell-200/80"
    } ${inputClassName}`,
  );
</script>

<div class="grid gap-1.5">
  <div class="flex items-start justify-between gap-3">
    <label class={labelClassName} for={id}>
      {label}{#if required}<span class="text-rose-600">*</span>{/if}
    </label>
    {#if error}
      <p id={errorId} class="min-w-0 text-right text-[12px] leading-5 text-rose-700">
        {error}
      </p>
    {/if}
  </div>

  <div class="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto]">
    <input
      aria-describedby={errorId}
      aria-invalid={invalid}
      class={inputClass}
      {disabled}
      {id}
      oninput={(event) => oninput(event.currentTarget.value)}
      {placeholder}
      required={inputRequired}
      type="text"
      {value}
    />
    <Button
      label={chooseLabel}
      icon={chooseIcon ?? undefined}
      variant={buttonVariant}
      className={buttonClassName}
      disabled={disabled || chooseDisabled}
      onclick={onchoose}
    />
  </div>

  {#if after}
    {@render after()}
  {/if}
</div>
