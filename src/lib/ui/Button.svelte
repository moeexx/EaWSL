<script module lang="ts">
  export type { ButtonSize, ButtonVariant } from "./button-styles";
</script>

<script lang="ts">
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import {
    buttonBaseClass,
    buttonSizeClassMap,
    buttonVariantClassMap,
    type ButtonSize,
    type ButtonVariant,
  } from "./button-styles";

  type IconComponent = typeof RefreshCw;

  type BaseProps = {
    iconStrokeWidth?: number;
    variant?: ButtonVariant;
    size?: ButtonSize;
    className?: string;
    disabled?: boolean;
    ariaControls?: string;
    ariaExpanded?: boolean;
    onclick: () => void | Promise<void>;
  };

  type Props =
    | (BaseProps & {
        icon: IconComponent;
        label: string;
        ariaLabel?: string;
      })
    | (BaseProps & {
        icon: IconComponent;
        label?: undefined;
        ariaLabel: string;
      });

  let {
    label = undefined,
    icon,
    iconStrokeWidth = 2,
    variant = "primary",
    size = "md",
    className = "",
    disabled = false,
    ariaLabel = undefined,
    ariaControls = undefined,
    ariaExpanded = undefined,
    onclick,
  }: Props = $props();

  const currentSizeClass = $derived(buttonSizeClassMap[size]);
  const CurrentIcon = $derived(icon);
  const iconOnly = $derived(!label);
  const currentIconSize = $derived(
    iconOnly ? currentSizeClass.iconOnlyIcon : currentSizeClass.icon,
  );
  const buttonSizeClass = $derived(
    iconOnly ? currentSizeClass.iconOnlyButton : currentSizeClass.button,
  );
</script>

<button
  aria-controls={ariaControls}
  aria-expanded={ariaExpanded}
  aria-label={ariaLabel}
  class={`${buttonBaseClass} ${buttonVariantClassMap[variant]} ${buttonSizeClass} ${className}`}
  {disabled}
  {onclick}
  type="button"
>
  <CurrentIcon
    class="shrink-0"
    size={currentIconSize}
    strokeWidth={iconStrokeWidth}
  />
  {#if label}
    <span>{label}</span>
  {/if}
</button>
