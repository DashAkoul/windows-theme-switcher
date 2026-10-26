// Types mirroring the Rust structs (serde camelCase).

export type ThemeMode = "light" | "dark";

export interface ThemeProfile {
  name: string;
  windowsMode: ThemeMode;
  appMode: ThemeMode;
  transparency: boolean;
  accentColor: string | null;
  wallpaper: string | null;
}

/** Only two settings remain in v2 (the panel is intentionally minimal). */
export interface Settings {
  startWithWindows: boolean;
  checkUpdates: boolean;
}

export interface AppConfig {
  version: number;
  settings: Settings;
  profiles: Record<string, ThemeProfile>;
  lastActiveProfile: string | null;
}

export interface ThemeState {
  appModeLight: boolean;
  systemModeLight: boolean;
  transparency: boolean;
  accentColor: string | null;
  wallpaper: string | null;
}

export interface AppStateResponse {
  config: AppConfig;
  current: ThemeState;
  version: string;
}

export const LIGHT_PROFILE_ID = "light";
export const DARK_PROFILE_ID = "dark";
