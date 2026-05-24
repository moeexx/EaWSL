<script lang="ts">
  import { afterNavigate, goto } from "$app/navigation";
  import { page } from "$app/state";

  import { i18nState } from "$lib/i18n";

  import ShellIcon from "./ShellIcon.svelte";
  import { shellUiState, toggleSidebarCollapsed } from "./state";

  type NavigationItem = {
    href: string;
    key: "acquire" | "distros" | "overview" | "settings";
    icon: "acquire" | "distros" | "monitor" | "settings";
  };

  const primaryNavigation = [
    { href: "/overview", key: "overview", icon: "monitor" },
    { href: "/distros", key: "distros", icon: "distros" },
    { href: "/acquire", key: "acquire", icon: "acquire" },
  ] satisfies NavigationItem[];

  const settingsNavigation = {
    href: "/settings",
    key: "settings",
    icon: "settings",
  } satisfies NavigationItem;

  const navigationItems = [...primaryNavigation, settingsNavigation];

  const sidebarClass = $derived(
    [
      "relative min-w-0 shrink-0 px-2.5 py-2.5 transition-[width] duration-200",
      $shellUiState.sidebarCollapsed ? "w-[70px]" : "w-[70px] md:w-[180px]",
    ].join(" "),
  );

  const sidebarToggleButtonClass =
    "flex h-10 w-10 items-center justify-center rounded-[8px] border-[0.5px] border-transparent text-shell-700 transition duration-150 hover:border-shell-200/80 hover:bg-white/[0.78] hover:text-shell-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-shell-300";

  let pendingNavigationHref = $state<string | null>(null);

  const shellCopy = $derived($i18nState.copy.shell);
  const currentPath = $derived(pendingNavigationHref ?? page.url.pathname);

  function isActive(href: string, path: string): boolean {
    return path === href || path.startsWith(`${href}/`);
  }

  function getNavigationLabel(item: NavigationItem): string {
    return shellCopy.navigation.items[item.key];
  }

  async function navigateTo(href: string): Promise<void> {
    if (isActive(href, currentPath)) {
      return;
    }

    pendingNavigationHref = href;

    try {
      await goto(href);
    } catch {
      pendingNavigationHref = null;
    }
  }

  function getNavButtonClass(active: boolean, collapsed: boolean): string {
    return [
      "group relative flex h-10 w-full items-center gap-2.5 overflow-hidden rounded-[8px] border-[0.5px] text-left transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-300/60",
      collapsed
        ? "justify-center px-0"
        : "justify-center px-2 md:justify-start md:px-3",
      active
        ? "border-accent-200 bg-accent-50 text-accent-700"
        : "border-transparent text-shell-700 hover:border-shell-200/80 hover:bg-white/[0.78] hover:text-shell-900",
    ].join(" ");
  }

  function getNavIconClass(active: boolean): string {
    return [
      "flex h-9 w-9 shrink-0 items-center justify-center rounded-[8px] transition duration-150",
      active ? "text-accent-700" : "text-shell-600 group-hover:text-shell-800",
    ].join(" ");
  }

  afterNavigate(() => {
    pendingNavigationHref = null;
  });
</script>

{#snippet navButton(item: NavigationItem)}
  {@const active = isActive(item.href, currentPath)}
  {@const label = getNavigationLabel(item)}

  <button
    aria-current={active ? "page" : undefined}
    class={getNavButtonClass(active, $shellUiState.sidebarCollapsed)}
    onclick={() => void navigateTo(item.href)}
    title={label}
    type="button"
  >
    <span class={getNavIconClass(active)}>
      <ShellIcon name={item.icon} size={20} />
    </span>

    <span
      class={$shellUiState.sidebarCollapsed
        ? "hidden"
        : "hidden min-w-0 flex-1 items-center overflow-hidden md:flex"}
    >
      <strong
        class="block truncate text-[14px] font-bold leading-5 text-shell-900"
      >
        {label}
      </strong>
    </span>
  </button>
{/snippet}

<aside class={sidebarClass}>
  <div class="flex h-full min-h-0 flex-col gap-3">
    <div
      class={`flex items-center px-0.5 ${$shellUiState.sidebarCollapsed ? "justify-center" : "justify-between"}`}
    >
      <p
        class={$shellUiState.sidebarCollapsed
          ? "hidden"
          : "eyebrow-label text-[13px]"}
      >
        {shellCopy.navigation.label}
      </p>
      <button
        aria-label={$shellUiState.sidebarCollapsed
          ? shellCopy.sidebar.expand
          : shellCopy.sidebar.collapse}
        class={sidebarToggleButtonClass}
        onclick={() => void toggleSidebarCollapsed()}
        title={$shellUiState.sidebarCollapsed
          ? shellCopy.sidebar.expand
          : shellCopy.sidebar.collapse}
        type="button"
      >
        <ShellIcon name="menu" size={20} />
      </button>
    </div>

    <nav
      aria-label={shellCopy.navigation.primaryAriaLabel}
      class="grid gap-1.5"
    >
      {#each primaryNavigation as item}
        {@render navButton(item)}
      {/each}
    </nav>

    <div class="mt-auto grid gap-1.5 pt-3">
      {@render navButton(settingsNavigation)}
    </div>
  </div>
</aside>
