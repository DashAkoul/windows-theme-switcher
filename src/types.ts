// TypeScript types mirroring the Rust structs (serde camelCase).

export type ThemeMode = "light" | "dark";
export type UiTheme = "system" | "light" | "dark";
export type Language = "en" | "fa";

export interface ThemeProfile {
  /** Display name, e.g. "Day" / "Night". */
  name: string;
  /** Taskbar, Start menu and system surfaces. */
  windowsMode: ThemeMode;
  /** Default theme for Windows apps. */
  appMode: ThemeMode;
  /** Window and taskbar transparency effects. */
  transparency: boolean;
  /** Accent color as #RRGGBB, or null to leave unchanged. */
  accentColor: string | null;
  /** Absolute wallpaper path, or null to leave unchanged. */
  wallpaper: string | null;
}

export interface Settings {
  startWithWindows: boolean;
  launchMinimized: boolean;
  checkUpdates: boolean;
  language: Language;
  uiTheme: UiTheme;
  accentColor: string;
  animations: boolean;
  transparency: boolean;
  onboardingCompleted: boolean;
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
  osBuild: number;
  isWindows11: boolean;
  osDisplay: string;
  exePath: string;
  logPath: string;
  pendingRoute: string | null;
}

export interface AppErrorPayload {
  code: string;
  message: string;
}

export const LIGHT_PROFILE_ID = "light";
export const DARK_PROFILE_ID = "dark";
