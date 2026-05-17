<script lang="ts">
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";

  import { i18nState } from "$lib/i18n";
  import Button, {
    type ButtonSize,
    type ButtonVariant,
  } from "$lib/ui/Button.svelte";

  type IconComponent = typeof RefreshCw;

  type Props = {
    label: string;
    icon?: IconComponent;
    refreshing: boolean;
    refreshIcon?: IconComponent;
    refreshingLabel?: string;
    color?: ButtonVariant;
    variant?: ButtonVariant;
    size?: ButtonSize;
    className?: string;
    disabled?: boolean;
    ariaControls?: string;
    ariaExpanded?: boolean;
    onclick: () => void;
  };

  let {
    label,
    icon,
    refreshing,
    refreshIcon,
    refreshingLabel,
    color = "primary",
    variant = color,
    size = "md",
    className = "",
    disabled = false,
    ariaControls = undefined,
    ariaExpanded = undefined,
    onclick,
  }: Props = $props();

  let CurrentIcon = $derived(
    refreshing ? (refreshIcon ?? RefreshCw) : (icon ?? RefreshCw),
  );
  const currentRefreshingLabel = $derived(
    refreshingLabel ?? $i18nState.copy.common.refreshing,
  );
</script>

<Button
  ariaBusy={refreshing}
  {ariaControls}
  {ariaExpanded}
  label={refreshing ? currentRefreshingLabel : label}
  icon={CurrentIcon}
  iconClass={refreshing ? "animate-spin" : ""}
  {variant}
  {size}
  {className}
  disabled={disabled || refreshing}
  {onclick}
/>
