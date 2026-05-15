import { invoke } from "@tauri-apps/api/core";
import { join, localDataDir } from "@tauri-apps/api/path";

export const DEFAULT_INSTALL_LOCATION = "C:\\WSL";
export const MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES = 1;
export const MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES = 1440;

const DEFAULT_INSTALL_DIR_NAME = "wsl";

export type BackgroundRefreshTarget =
  | "distros"
  | "systemOverviewStorage"
  | "wslVersion"
  | "onlineDistros";

export interface BackgroundRefreshSettings {
  intervalMinutes: number;
  targets: BackgroundRefreshTarget[];
}

export const BACKGROUND_REFRESH_TARGET_ORDER: BackgroundRefreshTarget[] = [
  "distros",
  "systemOverviewStorage",
  "wslVersion",
  "onlineDistros",
];

export const DEFAULT_BACKGROUND_REFRESH_SETTINGS: BackgroundRefreshSettings = {
  intervalMinutes: 15,
  targets: ["distros", "systemOverviewStorage", "wslVersion"],
};

export interface AppSettings {
  defaultInstallLocation: string;
  backgroundRefresh: BackgroundRefreshSettings;
}

export const DEFAULT_APP_SETTINGS: AppSettings = createDefaultAppSettings(
  DEFAULT_INSTALL_LOCATION,
);

export async function getAppSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_app_settings");
}

export async function saveAppSettings(
  settings: AppSettings,
): Promise<AppSettings> {
  const saved = await invoke<AppSettings>("save_app_settings", {
    settings: normalizeAppSettings(settings),
  });
  return normalizeAppSettings(saved);
}

export async function readAppSettingsOrDefault(): Promise<AppSettings> {
  try {
    return normalizeAppSettings(await getAppSettings());
  } catch {
    return normalizeAppSettings(
      createDefaultAppSettings(await resolveDefaultInstallLocation()),
    );
  }
}

export function createDefaultAppSettings(
  defaultInstallLocation: string,
): AppSettings {
  return {
    defaultInstallLocation,
    backgroundRefresh: DEFAULT_BACKGROUND_REFRESH_SETTINGS,
  };
}

export async function resolveDefaultInstallLocation(): Promise<string> {
  try {
    return await join(await localDataDir(), DEFAULT_INSTALL_DIR_NAME);
  } catch {
    return DEFAULT_INSTALL_LOCATION;
  }
}

export function normalizeAppSettings(settings: AppSettings): AppSettings {
  return {
    defaultInstallLocation: settings.defaultInstallLocation.trim(),
    backgroundRefresh: {
      intervalMinutes: settings.backgroundRefresh.intervalMinutes,
      targets: orderBackgroundRefreshTargets(settings.backgroundRefresh.targets),
    },
  };
}

export async function getDefaultInstallLocation(): Promise<string> {
  const settings = await readAppSettingsOrDefault();
  return settings.defaultInstallLocation || (await resolveDefaultInstallLocation());
}

export async function getBackgroundRefreshSettings(): Promise<BackgroundRefreshSettings> {
  const settings = await readAppSettingsOrDefault();
  return settings.backgroundRefresh;
}

export function orderBackgroundRefreshTargets(
  targets: readonly BackgroundRefreshTarget[],
): BackgroundRefreshTarget[] {
  return BACKGROUND_REFRESH_TARGET_ORDER.filter((target) =>
    targets.includes(target),
  );
}
