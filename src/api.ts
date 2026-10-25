// Typed wrappers around Tauri commands and events.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type {
  AppConfig,
  AppErrorPayload,
  AppStateResponse,
  ThemeProfile,
  ThemeState,
} from "./types";

function unwrapError(err: unknown): AppErrorPayload {
  if (typeof err === "object" && err !== null && "message" in err) {
    const e = err as { code?: string; message?: string };
    return { code: e.code ?? "unknown", message: e.message ?? String(err) };
  }
  return { code: "unknown", message: String(err) };
}

export const api = {
  getState(): Promise<AppStateResponse> {
    return invoke<AppStateResponse>("get_app_state").catch((e) => {
      throw unwrapError(e);
    });
  },
  toggleTheme(): Promise<ThemeState> {
    return invoke<ThemeState>("toggle_theme").catch((e) => {
      throw unwrapError(e);
    });
  },
  applyProfile(id: string): Promise<ThemeState> {
    return invoke<ThemeState>("apply_profile", { id }).catch((e) => {
      throw unwrapError(e);
    });
  },
  saveConfig(config: AppConfig): Promise<AppConfig> {
    return invoke<AppConfig>("save_config", { config }).catch((e) => {
      throw unwrapError(e);
    });
  },
  setAutostart(enabled: boolean): Promise<boolean> {
    return invoke<boolean>("set_autostart", { enabled }).catch((e) => {
      throw unwrapError(e);
    });
  },
  detectCurrentProfile(): Promise<ThemeProfile> {
    return invoke<ThemeProfile>("detect_current_profile").catch((e) => {
      throw unwrapError(e);
    });
  },
  openWindowsSettings(): Promise<void> {
    return invoke<void>("open_windows_settings");
  },
  openUrl(url: string): Promise<void> {
    return invoke<void>("open_url", { url });
  },
  getDiagnostics(): Promise<string> {
    return invoke<string>("get_diagnostics");
  },
  clearPendingRoute(): Promise<void> {
    return invoke<void>("clear_pending_route");
  },
  resetConfig(): Promise<AppConfig> {
    return invoke<AppConfig>("reset_config").catch((e) => {
      throw unwrapError(e);
    });
  },
  configPath(): Promise<string> {
    return invoke<string>("config_path");
  },
};

/** Open a native file picker for wallpaper images. */
export async function pickWallpaper(): Promise<string | null> {
  const result = await openDialog({
    multiple: false,
    directory: false,
    filters: [
      { name: "Images", extensions: ["jpg", "jpeg", "png", "bmp"] },
    ],
  });
  if (typeof result === "string") return result;
  return null;
}

/** Subscribe to Windows-originated theme changes. */
export function onSystemThemeChanged(cb: (state: ThemeState) => void): Promise<() => void> {
  return listen<ThemeState>("system-theme-changed", (e) => cb(e.payload));
}

/** Subscribe to theme applications done by this app. */
export function onThemeApplied(cb: (state: ThemeState) => void): Promise<() => void> {
  return listen<ThemeState>("theme-applied", (e) => cb(e.payload));
}

/** Subscribe to theme errors (shown as toasts). */
export function onThemeError(cb: (payload: AppErrorPayload) => void): Promise<() => void> {
  return listen<AppErrorPayload>("theme-error", (e) => cb(e.payload));
}

/** Subscribe to tray navigation requests. */
export function onTrayNavigate(cb: (route: string) => void): Promise<() => void> {
  return listen<string>("tray://navigate", (e) => cb(e.payload));
}

/** Copy text to clipboard with a fallback for locked-down webviews. */
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      const ta = document.createElement("textarea");
      ta.value = text;
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      const ok = document.execCommand("copy");
      document.body.removeChild(ta);
      return ok;
    } catch {
      return false;
    }
  }
}
