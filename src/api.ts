// Thin typed wrappers around the Tauri commands.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AppConfig, AppStateResponse, Settings, ThemeState } from "./types";

/** True when running inside the Tauri webview (not a plain browser). */
export const isTauri = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Mock state so the panel can be previewed in a browser. */
function mockState(): AppStateResponse {
  return {
    config: {
      version: 1,
      settings: { startWithWindows: false, checkUpdates: true },
      profiles: {
        light: {
          name: "Day",
          windowsMode: "light",
          appMode: "light",
          transparency: true,
          accentColor: null,
          wallpaper: null,
        },
        dark: {
          name: "Night",
          windowsMode: "dark",
          appMode: "dark",
          transparency: true,
          accentColor: null,
          wallpaper: null,
        },
      },
      lastActiveProfile: null,
    },
    current: {
      appModeLight: false,
      systemModeLight: false,
      transparency: true,
      accentColor: null,
      wallpaper: null,
    },
    isWindows11: true,
    version: "2.0.0",
  };
}

export const api = {
  getState(): Promise<AppStateResponse> {
    if (!isTauri()) return Promise.resolve(mockState());
    return invoke<AppStateResponse>("get_app_state");
  },
  applyProfile(id: string): Promise<ThemeState> {
    if (!isTauri()) return Promise.resolve(mockState().current);
    return invoke<ThemeState>("apply_profile", { id });
  },
  saveSettings(settings: Settings): Promise<AppConfig> {
    if (!isTauri()) return Promise.resolve(mockState().config);
    return invoke<AppConfig>("save_settings", { settings });
  },
  openWindowsSettings(): Promise<void> {
    if (!isTauri()) return Promise.resolve();
    return invoke<void>("open_windows_settings");
  },
  hidePanel(): Promise<void> {
    if (!isTauri()) return Promise.resolve();
    return invoke<void>("hide_panel");
  },
};

/** Close (destroy) the panel window. */
export function closePanel(): void {
  if (!isTauri()) return;
  void getCurrentWindow().close();
}

/** Subscribe to theme changes applied by this app or by Windows. */
export function onThemeChanged(cb: (state: ThemeState) => void): Promise<() => void> {
  if (!isTauri()) return Promise.resolve(() => {});
  const offs: (() => void)[] = [];
  const a = listen<ThemeState>("theme-applied", (e) => cb(e.payload)).then((off) => {
    offs.push(off);
  });
  const b = listen<ThemeState>("system-theme-changed", (e) => cb(e.payload)).then((off) => {
    offs.push(off);
  });
  return Promise.all([a, b]).then(() => () => offs.forEach((off) => off()));
}

/** Report a human-readable error message from a command rejection. */
export function errorMessage(err: unknown): string {
  if (typeof err === "object" && err !== null && "message" in err) {
    const m = (err as { message?: unknown }).message;
    if (typeof m === "string" && m.length > 0) return m;
  }
  return String(err);
}
