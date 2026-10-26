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

/** True when running inside the Tauri webview (not a plain browser). */
export const isTauri = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * Mock state used when the UI is opened in a regular browser
 * (`npm run dev` / `npm run preview`). Keeps the whole UI explorable
 * without the native layer.
 */
function mockState(): AppStateResponse {
  const light: ThemeProfile = {
    name: "Day",
    windowsMode: "light",
    appMode: "light",
    transparency: true,
    accentColor: null,
    wallpaper: null,
  };
  const dark: ThemeProfile = {
    name: "Night",
    windowsMode: "dark",
    appMode: "dark",
    transparency: true,
    accentColor: null,
    wallpaper: null,
  };
  const config: AppConfig = {
    version: 1,
    settings: {
      startWithWindows: false,
      launchMinimized: true,
      checkUpdates: true,
      language: "en",
      uiTheme: "system",
      accentColor: "#0078D4",
      animations: true,
      transparency: true,
      onboardingCompleted: false,
    },
    profiles: { light, dark },
    lastActiveProfile: null,
  };
  return {
    config,
    current: {
      appModeLight: false,
      systemModeLight: false,
      transparency: true,
      accentColor: null,
      wallpaper: null,
    },
    osBuild: 26300,
    isWindows11: true,
    osDisplay: "Windows 11 (browser preview)",
    exePath: "C:\\mock\\windows-theme-switcher.exe",
    logPath: "C:\\mock\\app.log",
    pendingRoute: null,
  };
}

function unwrapError(err: unknown): AppErrorPayload {
  if (typeof err === "object" && err !== null && "message" in err) {
    const e = err as { code?: string; message?: string };
    return { code: e.code ?? "unknown", message: e.message ?? String(err) };
  }
  return { code: "unknown", message: String(err) };
}

export const api = {
  getState(): Promise<AppStateResponse> {
    if (!isTauri()) return Promise.resolve(mockState());
    return invoke<AppStateResponse>("get_app_state").catch((e) => {
      throw unwrapError(e);
    });
  },
  toggleTheme(): Promise<ThemeState> {
    if (!isTauri()) {
      return Promise.resolve(mockState().current);
    }
    return invoke<ThemeState>("toggle_theme").catch((e) => {
      throw unwrapError(e);
    });
  },
  applyProfile(id: string): Promise<ThemeState> {
    if (!isTauri()) return Promise.resolve(mockState().current);
    return invoke<ThemeState>("apply_profile", { id }).catch((e) => {
      throw unwrapError(e);
    });
  },
  saveConfig(config: AppConfig): Promise<AppConfig> {
    if (!isTauri()) return Promise.resolve(config);
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
    if (!isTauri()) return Promise.resolve(mockState().config.profiles.light);
    return invoke<ThemeProfile>("detect_current_profile").catch((e) => {
      throw unwrapError(e);
    });
  },
  openWindowsSettings(): Promise<void> {
    if (!isTauri()) return Promise.resolve();
    return invoke<void>("open_windows_settings");
  },
  openUrl(url: string): Promise<void> {
    if (!isTauri()) {
      window.open(url, "_blank", "noopener,noreferrer");
      return Promise.resolve();
    }
    return invoke<void>("open_url", { url });
  },
  getDiagnostics(): Promise<string> {
    if (!isTauri()) return Promise.resolve(JSON.stringify(mockState(), null, 2));
    return invoke<string>("get_diagnostics");
  },
  clearPendingRoute(): Promise<void> {
    if (!isTauri()) return Promise.resolve();
    return invoke<void>("clear_pending_route");
  },
  resetConfig(): Promise<AppConfig> {
    if (!isTauri()) return Promise.resolve(mockState().config);
    return invoke<AppConfig>("reset_config").catch((e) => {
      throw unwrapError(e);
    });
  },
  configPath(): Promise<string> {
    if (!isTauri()) return Promise.resolve("C:\\mock\\config.json");
    return invoke<string>("config_path");
  },
};

/** Subscribe helper that is a no-op outside Tauri. */
function safeListen<T>(event: string, cb: (payload: T) => void): Promise<() => void> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<T>(event, (e) => cb(e.payload));
}

/** Open a native file picker for wallpaper images. */
export async function pickWallpaper(): Promise<string | null> {
  if (!isTauri()) return null;
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
  return safeListen<ThemeState>("system-theme-changed", cb);
}

/** Subscribe to theme applications done by this app. */
export function onThemeApplied(cb: (state: ThemeState) => void): Promise<() => void> {
  return safeListen<ThemeState>("theme-applied", cb);
}

/** Subscribe to theme errors (shown as toasts). */
export function onThemeError(cb: (payload: AppErrorPayload) => void): Promise<() => void> {
  return safeListen<AppErrorPayload>("theme-error", cb);
}

/** Subscribe to tray navigation requests. */
export function onTrayNavigate(cb: (route: string) => void): Promise<() => void> {
  return safeListen<string>("tray://navigate", cb);
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
