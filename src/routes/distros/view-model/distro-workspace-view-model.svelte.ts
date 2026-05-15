import { untrack } from "svelte";

import { getCopy, i18nState } from "$lib/i18n";
import { longTaskState } from "$lib/long-tasks";
import { findDistroByName } from "$lib/shared/distros";
import type { DistroInfo } from "$lib/tauri/wsl";

import { createDistroWorkspaceService, type DistroWorkspaceService } from "../service/distro-workspace-service";
import {
  createDistroWorkspaceActions,
  type DistroWorkspaceActions,
} from "./distro-workspace-actions";
import { buildDistroWorkspaceView } from "./distro-workspace-display";
import type {
  DistroWorkspaceActionButtonKey,
  DistroWorkspaceCallbacks,
  DistroWorkspaceOverlayState,
  DistroWorkspaceView,
  ShutdownMode,
} from "./distro-workspace-types";

export interface DistroWorkspaceViewModel {
  readonly view: DistroWorkspaceView;
  readonly callbacks: DistroWorkspaceCallbacks;
}

export function createDistroWorkspaceViewModel(
  service: DistroWorkspaceService = createDistroWorkspaceService(),
): DistroWorkspaceViewModel {
  let queryState = $state(service.getQueryState());
  let vhdSizeState = $state(service.getVhdSizeState());
  let workspaceRefreshing = $state(false);
  let activeActionButtonKey = $state<DistroWorkspaceActionButtonKey>(null);
  let expandedPanels = $state<Record<string, boolean>>({});
  let actionOverlays = $state<DistroWorkspaceOverlayState[]>([]);
  let shutdownMode = $state<ShutdownMode>(null);
  let hasActiveLongTask = $state(false);
  let copy = $state(getCopy());
  let lastDistrosRef = queryState.distros.data;
  let disposed = false;

  const view = $derived(
    buildDistroWorkspaceView({
      copy,
      queryState,
      vhdSizeState,
      workspaceRefreshing,
      activeActionButtonKey,
      expandedPanels,
      actionOverlays,
      shutdownMode,
      hasActiveLongTask,
      getVhdSizeEntry: service.getVhdSizeEntry,
      getInstallLocation: service.getInstallLocation,
    }),
  );

  const actions: DistroWorkspaceActions = createDistroWorkspaceActions({
    service,
    getCopy: () => copy,
    isDisposed: () => disposed,
    getActionOverlays: () => actionOverlays,
    setActionOverlays: (nextOverlays) => {
      actionOverlays = nextOverlays;
    },
    setShutdownMode: (mode) => {
      shutdownMode = mode;
    },
    setActiveActionButtonKey: (key) => {
      activeActionButtonKey = key;
    },
  });

  function reconcileExpandedPanels(distros: DistroInfo[]): void {
    const availableNames = new Set(distros.map((distro) => distro.name));
    const nextPanels = Object.fromEntries(
      Object.entries(expandedPanels).filter(([name]) => availableNames.has(name)),
    );

    expandedPanels = nextPanels;
    hydrateExpandedPanels(distros, nextPanels);
  }

  function hydrateExpandedPanels(
    distros: DistroInfo[],
    panels: Record<string, boolean>,
  ): void {
    for (const distro of distros) {
      if (panels[distro.name] !== true) {
        continue;
      }

      const cachedEntry = vhdSizeState[distro.name];
      const entry = service.getVhdSizeEntry(vhdSizeState, distro);

      if (entry.status === "idle" || cachedEntry === undefined) {
        service.probeVhdSize(distro);
      }
    }
  }

  function setExpandedPanel(distroName: string, expanded: boolean): void {
    if (!expanded) {
      const nextPanels = { ...expandedPanels };
      delete nextPanels[distroName];
      expandedPanels = nextPanels;
      return;
    }

    expandedPanels = {
      ...expandedPanels,
      [distroName]: expanded,
    };
  }

  async function toggleExpanded(distroName: string): Promise<void> {
    if (expandedPanels[distroName] === true) {
      setExpandedPanel(distroName, false);
      return;
    }

    const latestDistro = findDistroByName(
      service.getQueryState().distros.data ?? [],
      distroName,
    );

    if (!latestDistro) {
      return;
    }

    if (!disposed) {
      setExpandedPanel(latestDistro.name, true);
      service.probeVhdSize(latestDistro);
    }
  }

  function handleQueryStateChanged(nextState: typeof queryState): void {
    queryState = nextState;

    if (nextState.distros.data !== lastDistrosRef) {
      lastDistrosRef = nextState.distros.data;
      reconcileExpandedPanels(nextState.distros.data ?? []);
    }
  }

  $effect(() => untrack(() => {
    disposed = false;
    void service.enterWorkspace();

    const unsubscribeQuery = service.queryCache.subscribe(handleQueryStateChanged);
    const unsubscribeI18n = i18nState.subscribe((state) => {
      copy = state.copy;
    });
    const unsubscribeLongTask = longTaskState.subscribe((state) => {
      hasActiveLongTask = state.hasActiveLongTask;
    });
    const unsubscribeVhdSize = service.vhdSizeCache.subscribe((state) => {
      untrack(() => {
        vhdSizeState = state;
        hydrateExpandedPanels(queryState.distros.data ?? [], expandedPanels);
      });
    });

    return () => {
      disposed = true;
      unsubscribeQuery();
      unsubscribeI18n();
      unsubscribeLongTask();
      unsubscribeVhdSize();
    };
  }));

  async function refreshWorkspace(): Promise<void> {
    workspaceRefreshing = true;

    try {
      await actions.refresh();
    } finally {
      workspaceRefreshing = false;
    }
  }

  const callbacks: DistroWorkspaceCallbacks = {
    ...actions,
    refresh: refreshWorkspace,
    toggleExpanded,
  };

  return {
    get view() {
      return view;
    },
    callbacks,
  };
}
