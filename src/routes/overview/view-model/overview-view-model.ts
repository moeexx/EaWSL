import {
  getQueryErrorMessage,
  getQueryRecoveringHint,
  type QueryCacheState,
} from "$lib/query-cache";
import type { AppCopy } from "$lib/i18n";

import {
  buildOverviewSystemCards,
  type OverviewSystemInfoCard,
} from "./system-overview-cards";
import { buildWslInfoItems, type InfoItem } from "./wsl-overview-display";

export type OverviewNotice = {
  key: string;
  tone: "error" | "warning";
  title: string;
  message: string;
};

export type OverviewInfoItem = InfoItem;

export type OverviewViewModel = {
  notices: OverviewNotice[];
  systemCards: OverviewSystemInfoCard[];
  wslInfoItems: OverviewInfoItem[];
};

type NoticeDefinition = {
  key: string;
  tone: OverviewNotice["tone"];
  title: (copy: AppCopy) => string;
  message: (state: QueryCacheState, copy: AppCopy) => string | null;
};

const noticeDefinitions: NoticeDefinition[] = [
  {
    key: "system-overview-error",
    tone: "error",
    title: (copy) => copy.overview.notices.systemError,
    message: (state, copy) =>
      state.systemOverview.hasError
        ? getQueryErrorMessage("systemOverview", state.systemOverview, copy)
        : null,
  },
  {
    key: "system-overview-recovering",
    tone: "warning",
    title: (copy) => copy.overview.notices.systemRecovering,
    message: (state, copy) =>
      getQueryRecoveringHint("systemOverview", state.systemOverview, copy),
  },
  {
    key: "wsl-version-error",
    tone: "error",
    title: (copy) => copy.overview.notices.wslError,
    message: (state, copy) =>
      state.wslVersion.hasError
        ? getQueryErrorMessage("wslVersion", state.wslVersion, copy)
        : null,
  },
  {
    key: "wsl-version-recovering",
    tone: "warning",
    title: (copy) => copy.overview.notices.wslRecovering,
    message: (state, copy) =>
      getQueryRecoveringHint("wslVersion", state.wslVersion, copy),
  },
  {
    key: "distros-recovering",
    tone: "warning",
    title: (copy) => copy.overview.notices.distrosRecovering,
    message: (state, copy) =>
      getQueryRecoveringHint("distros", state.distros, copy),
  },
];

export function buildOverviewViewModel(
  state: QueryCacheState,
  copy: AppCopy,
): OverviewViewModel {
  const isSystemPending =
    state.systemOverview.activity === "loading" &&
    state.systemOverview.data === null;

  return {
    notices: buildOverviewNotices(state, copy),
    systemCards: buildOverviewSystemCards(
      state.systemOverview.data,
      isSystemPending,
      copy,
    ),
    wslInfoItems: buildWslInfoItems(state, copy),
  };
}

function buildOverviewNotices(
  state: QueryCacheState,
  copy: AppCopy,
): OverviewNotice[] {
  return noticeDefinitions.flatMap((definition) => {
    const message = definition.message(state, copy);

    if (!message) {
      return [];
    }

    return [
      {
        key: definition.key,
        tone: definition.tone,
        title: definition.title(copy),
        message,
      },
    ];
  });
}
