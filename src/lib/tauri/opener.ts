import { openUrl as tauriOpenUrl } from "@tauri-apps/plugin-opener";

import { normalizeTauriCommandError } from "./errors";

export async function openUrl(url: string): Promise<void> {
  try {
    await tauriOpenUrl(url);
  } catch (error) {
    throw normalizeTauriCommandError(error);
  }
}
