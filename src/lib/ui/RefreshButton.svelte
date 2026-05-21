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

  type BaseProps = {
    icon?: IconComponent;
    refreshing: boolean;
    refreshIcon?: IconComponent;
    refreshingLabel?: string;
    variant?: ButtonVariant;
    size?: ButtonSize;
    className?: string;
    disabled?: boolean;
    onclick: () => void | Promise<void>;
  };

  type Props =
    | (BaseProps & {
        label: string;
        ariaLabel?: string;
      })
    | (BaseProps & {
        label?: undefined;
        ariaLabel: string;
      });

  let {
    label = undefined,
    icon,
    refreshing,
    refreshIcon,
    refreshingLabel,
    variant = "primary",
    size = "md",
    className = "",
    disabled = false,
    ariaLabel = undefined,
    onclick,
  }: Props = $props();

  const currentSizeClass = $derived(buttonSizeClassMap[size]);
  const CurrentIcon = $derived(
    refreshing ? (refreshIcon ?? RefreshCw) : (icon ?? RefreshCw),
  );
  const currentLabel = $derived(
    label === undefined
      ? undefined
      : refreshing
        ? (refreshingLabel ?? $i18nState.copy.common.refreshing)
        : label,
  );
  const iconOnly = $derived(currentLabel === undefined);
  const buttonSizeClass = $derived(
    iconOnly ? currentSizeClass.iconOnlyButton : currentSizeClass.button,
  );
  const currentIconSize = $derived(
    iconOnly ? currentSizeClass.iconOnlyIcon : currentSizeClass.icon,
  );
</script>

<button
  aria-busy={refreshing}
  aria-label={ariaLabel}
  class={`${buttonBaseClass} ${buttonVariantClassMap[variant]} ${buttonSizeClass} ${className}`}
  disabled={disabled || refreshing}
  {onclick}
  type="button"
>
  <CurrentIcon
    class={`shrink-0 ${refreshing ? "animate-spin" : ""}`}
    size={currentIconSize}
    strokeWidth={2}
  />
  {#if currentLabel}
    <span>{currentLabel}</span>
  {/if}
</button>
