<script module lang="ts">
  export type { ButtonSize, ButtonVariant } from "./button-styles";
</script>

<script lang="ts">
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import type { Snippet } from "svelte";
  import {
    buttonBaseClass,
    buttonSizeClassMap,
    buttonVariantClassMap,
    type ButtonSize,
    type ButtonVariant,
  } from "./button-styles";

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
    onclick: () => void | Promise<void>;
  };

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

  const currentSizeClass = $derived(buttonSizeClassMap[size]);
  const CurrentIcon = $derived(icon);
  const iconOnly = $derived(!label && !children && Boolean(CurrentIcon));
  const currentIconSize = $derived(
    iconSize ??
      (iconOnly ? currentSizeClass.iconOnlyIcon : currentSizeClass.icon),
  );
  const currentIconStyle = $derived(
    `width: ${currentIconSize}px; height: ${currentIconSize}px;`,
  );
  const buttonSizeClass = $derived(
    children
      ? ""
      : iconOnly
        ? currentSizeClass.iconOnlyButton
        : currentSizeClass.button,
  );
</script>

<button
  aria-busy={ariaBusy}
  aria-controls={ariaControls}
  aria-expanded={ariaExpanded}
  aria-label={ariaLabel}
  aria-pressed={ariaPressed}
  class={`${buttonBaseClass} ${buttonVariantClassMap[variant]} ${buttonSizeClass} ${className}`}
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
        class={`shrink-0 ${iconClass}`}
        size={currentIconSize}
        style={currentIconStyle}
        strokeWidth={iconStrokeWidth}
      />
    {/if}
    {#if label}
      <span>{label}</span>
    {/if}
  {/if}
</button>
