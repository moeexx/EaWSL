<script lang="ts">
  import AppWindow from "@lucide/svelte/icons/app-window";
  import Boxes from "@lucide/svelte/icons/boxes";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronUp from "@lucide/svelte/icons/chevron-up";
  import Download from "@lucide/svelte/icons/download";
  import House from "@lucide/svelte/icons/house";
  import ListTodo from "@lucide/svelte/icons/list-todo";
  import Menu from "@lucide/svelte/icons/menu";
  import Minus from "@lucide/svelte/icons/minus";
  import Monitor from "@lucide/svelte/icons/monitor";
  import Settings from "@lucide/svelte/icons/settings";
  import Square from "@lucide/svelte/icons/square";
  import SquareSquare from "@lucide/svelte/icons/square-square";
  import X from "@lucide/svelte/icons/x";

  const icons = {
    app: AppWindow,
    menu: Menu,
    home: House,
    monitor: Monitor,
    distros: Boxes,
    acquire: Download,
    settings: Settings,
    minimize: Minus,
    maximize: Square,
    restore: SquareSquare,
    close: X,
    tasks: ListTodo,
    "chevron-up": ChevronUp,
    "chevron-down": ChevronDown,
  } as const;

  type IconName = keyof typeof icons;

  let {
    name = "app",
    size = 18,
    stroke = 1.8,
  }: {
    name?: IconName;
    size?: number;
    stroke?: number;
  } = $props();

  const Icon = $derived(icons[name]);
</script>

<Icon
  class="block shrink-0"
  aria-hidden="true"
  focusable="false"
  {size}
  strokeWidth={stroke}
/>
