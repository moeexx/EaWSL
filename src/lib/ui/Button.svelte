<script module lang="ts">
  export type ButtonVariant =
    | "primary"
    | "secondary"
    | "success"
    | "warning"
    | "danger";
  export type ButtonSize = "sm" | "md" | "lg";
</script>

<script lang="ts">
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import type { Snippet } from "svelte";

  type IconComponent = typeof RefreshCw;

  type Props = {
    label?: string;
    icon?: IconComponent;
    iconClass?: string;
    iconSize?: number;
    iconStrokeWidth?: number;
    variant?: ButtonVariant;
    size?: ButtonSize;
    className?: string;
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
    title?: string;
    ariaLabel?: string;
    ariaPressed?: boolean | "true" | "false" | "mixed";
    ariaControls?: string;
    ariaExpanded?: boolean;
    ariaBusy?: boolean;
    children?: Snippet;
    onclick: () => void;
  };

  const variantClassMap = {
    primary:
      "border-accent-600 bg-accent-600 text-white hover:border-accent-700 hover:bg-accent-700 focus-visible:ring-accent-300",
    secondary:
      "border-shell-200/80 bg-white/90 text-shell-700 hover:border-shell-300 hover:bg-shell-50 focus-visible:ring-shell-300",
    success:
      "border-emerald-600 bg-emerald-600 text-white hover:border-emerald-700 hover:bg-emerald-700 focus-visible:ring-emerald-300",
    warning:
      "border-amber-500 bg-amber-500 text-white hover:border-amber-600 hover:bg-amber-600 focus-visible:ring-amber-300",
    danger:
      "border-rose-600 bg-rose-600 text-white hover:border-rose-700 hover:bg-rose-700 focus-visible:ring-rose-300",
  } as const;

  const sizeClassMap = {
    sm: {
      button: "min-h-[30px] gap-1.5 px-2.5 py-1.5 text-[12px]",
      icon: 14,
    },
    md: {
      button: "min-h-[33px] gap-2 px-3 py-1.5 text-[13px]",
      icon: 16,
    },
    lg: {
      button: "min-h-[42px] gap-2.5 px-4 py-2.5 text-[14px]",
      icon: 18,
    },
  } as const;

  let {
    label,
    icon,
    iconClass = "",
    iconSize,
    iconStrokeWidth = 2,
    variant = "primary",
    size = "md",
    className = "",
    disabled = false,
    type = "button",
    title = undefined,
    ariaLabel = undefined,
    ariaPressed = undefined,
    ariaControls = undefined,
    ariaExpanded = undefined,
    ariaBusy = undefined,
    children,
    onclick,
  }: Props = $props();

  const baseButtonClass =
    "inline-flex cursor-pointer items-center justify-center rounded-[8px] border-[0.5px] font-semibold transition duration-150 focus-visible:outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:opacity-60";

  const currentSizeClass = $derived(sizeClassMap[size]);
  const CurrentIcon = $derived(icon);
  const currentIconSize = $derived(iconSize ?? currentSizeClass.icon);
</script>

<button
  aria-busy={ariaBusy}
  aria-controls={ariaControls}
  aria-expanded={ariaExpanded}
  aria-label={ariaLabel}
  aria-pressed={ariaPressed}
  class={`${baseButtonClass} ${variantClassMap[variant]} ${currentSizeClass.button} ${className}`}
  {disabled}
  {onclick}
  {title}
  {type}
>
  {#if children}
    {@render children()}
  {:else}
    {#if CurrentIcon}
      <CurrentIcon
        class={iconClass}
        size={currentIconSize}
        strokeWidth={iconStrokeWidth}
      />
    {/if}
    {#if label}
      <span>{label}</span>
    {/if}
  {/if}
</button>
