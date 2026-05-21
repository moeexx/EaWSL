<script lang="ts">
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";

  import { i18nState } from "$lib/i18n";
  import {
    buttonBaseClass,
    buttonSizeClassMap,
    buttonVariantClassMap,
    type ButtonSize,
    type ButtonVariant,
  } from "$lib/ui/button-styles";

  type IconComponent = typeof RefreshCw;

  type Props = {
    label: string;
    icon?: IconComponent;
    refreshing: boolean;
    refreshIcon?: IconComponent;
    refreshingLabel?: string;
    variant?: ButtonVariant;
    size?: ButtonSize;
    className?: string;
    disabled?: boolean;
    ariaControls?: string;
    ariaExpanded?: boolean;
    onclick: () => void | Promise<void>;
  };

  let {
    label,
    icon,
    refreshing,
    refreshIcon,
    refreshingLabel,
    variant = "primary",
    size = "md",
    className = "",
    disabled = false,
    ariaControls = undefined,
    ariaExpanded = undefined,
    onclick,
  }: Props = $props();

  const currentSizeClass = $derived(buttonSizeClassMap[size]);
  const CurrentIcon = $derived(
    refreshing ? (refreshIcon ?? RefreshCw) : (icon ?? RefreshCw),
  );
  const currentLabel = $derived(
    refreshing
      ? (refreshingLabel ?? $i18nState.copy.common.refreshing)
      : label,
  );
  const currentIconStyle = $derived(
    `width: ${currentSizeClass.icon}px; height: ${currentSizeClass.icon}px;`,
  );
</script>

<button
  aria-busy={refreshing}
  aria-controls={ariaControls}
  aria-expanded={ariaExpanded}
  class={`${buttonBaseClass} ${buttonVariantClassMap[variant]} ${currentSizeClass.button} ${className}`}
  disabled={disabled || refreshing}
  {onclick}
  type="button"
>
  <CurrentIcon
    class={`shrink-0 ${refreshing ? "animate-spin" : ""}`}
    size={currentSizeClass.icon}
    style={currentIconStyle}
    strokeWidth={2}
  />
  <span>{currentLabel}</span>
</button>
