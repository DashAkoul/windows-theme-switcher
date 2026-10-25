// Settings window shell: sidebar navigation + all sections.

import { useState, useEffect } from "react";
import { useI18n } from "../i18n";
import { api, copyText } from "../api";
import { ColorField, Segmented, SettingRow, Toggle, MoonIcon, SunIcon } from "../components/controls";
import { ThemeEditor } from "./ThemeEditor";
import type {
  AppConfig,
  AppStateResponse,
  Language,
  Settings,
  ThemeProfile,
  UiTheme,
} from "../types";
import { DARK_PROFILE_ID, LIGHT_PROFILE_ID } from "../types";

export type Section = "general" | "light" | "dark" | "appearance" | "advanced";

interface Props {
  state: AppStateResponse;
  onConfigChange: (config: AppConfig) => void;
  onError: (message: string) => void;
  onCurrentChange: (state: AppStateResponse["current"]) => void;
}

export function SettingsView({ state, onConfigChange, onError, onCurrentChange }: Props) {
  const { t } = useI18n();
  const [section, setSection] = useState<Section>("general");
  const config = state.config;

  const updateSettings = async (patch: Partial<Settings>) => {
    const next: AppConfig = {
      ...config,
      settings: { ...config.settings, ...patch },
    };
    try {
      const saved = await api.saveConfig(next);
      onConfigChange(saved);
    } catch (e) {
      onError((e as { message?: string }).message ?? String(e));
    }
  };

  const updateProfile = async (id: string, profile: ThemeProfile) => {
    const next: AppConfig = {
      ...config,
      profiles: { ...config.profiles, [id]: profile },
    };
    try {
      const saved = await api.saveConfig(next);
      onConfigChange(saved);
    } catch (e) {
      onError((e as { message?: string }).message ?? String(e));
    }
  };

  const navItems: { id: Section; label: string; theme?: "light" | "dark" }[] = [
    { id: "general", label: t("nav.general") },
    { id: "light", label: t("nav.light"), theme: "light" },
    { id: "dark", label: t("nav.dark"), theme: "dark" },
    { id: "appearance", label: t("nav.appearance") },
    { id: "advanced", label: t("nav.advanced") },
  ];

  return (
    <div className="app-body">
      <nav className="sidebar">
        {navItems.map((item) => (
          <button
            key={item.id}
            className={`nav-item${section === item.id ? " active" : ""}`}
            onClick={() => setSection(item.id)}
          >
            {item.label}
            {item.theme && (
              <span className={`nav-badge status-dot ${item.theme}`} />
            )}
          </button>
        ))}
      </nav>

      <div className="content">
        {section === "general" && (
          <div className="anim">
            <CurrentThemeBanner state={state} />
            <h2 className="page-title">{t("nav.general")}</h2>
            <p className="page-subtitle">{t("app.tagline")}</p>

            <SettingRow
              title={t("general.startWithWindows")}
              desc={t("general.startWithWindows.desc")}
            >
              <Toggle
                checked={config.settings.startWithWindows}
                label={t("general.startWithWindows")}
                onChange={(v) => updateSettings({ startWithWindows: v })}
              />
            </SettingRow>

            <SettingRow
              title={t("general.launchMinimized")}
              desc={t("general.launchMinimized.desc")}
            >
              <Toggle
                checked={config.settings.launchMinimized}
                label={t("general.launchMinimized")}
                onChange={(v) => updateSettings({ launchMinimized: v })}
              />
            </SettingRow>

            <SettingRow
              title={t("general.checkUpdates")}
              desc={t("general.checkUpdates.desc")}
            >
              <Toggle
                checked={config.settings.checkUpdates}
                label={t("general.checkUpdates")}
                onChange={(v) => updateSettings({ checkUpdates: v })}
              />
            </SettingRow>

            <div className="card">
              <div className="card-row">
                <div className="card-row-text">
                  <p className="card-title">{t("general.language")}</p>
                </div>
                <select
                  className="select"
                  value={config.settings.language}
                  onChange={(e) => updateSettings({ language: e.target.value as Language })}
                >
                  <option value="en">English</option>
                  <option value="fa">فارسی (Persian)</option>
                </select>
              </div>
            </div>
          </div>
        )}

        {section === "light" && (
          <div className="anim">
            <h2 className="page-title">{t("nav.light")}</h2>
            <p className="page-subtitle">{t("preview.hint")}</p>
            <ThemeEditor
              key={LIGHT_PROFILE_ID}
              profileId={LIGHT_PROFILE_ID}
              profile={config.profiles[LIGHT_PROFILE_ID]}
              onSave={(p) => updateProfile(LIGHT_PROFILE_ID, p)}
              onApplied={() => onCurrentChange(state.current)}
              onError={onError}
            />
          </div>
        )}

        {section === "dark" && (
          <div className="anim">
            <h2 className="page-title">{t("nav.dark")}</h2>
            <p className="page-subtitle">{t("preview.hint")}</p>
            <ThemeEditor
              key={DARK_PROFILE_ID}
              profileId={DARK_PROFILE_ID}
              profile={config.profiles[DARK_PROFILE_ID]}
              onSave={(p) => updateProfile(DARK_PROFILE_ID, p)}
              onApplied={() => onCurrentChange(state.current)}
              onError={onError}
            />
          </div>
        )}

        {section === "appearance" && (
          <div className="anim">
            <h2 className="page-title">{t("nav.appearance")}</h2>
            <p className="page-subtitle">{t("app.tagline")}</p>

            <div className="card">
              <div className="card-row">
                <div className="card-row-text">
                  <p className="card-title">{t("appearance.uiTheme")}</p>
                  <p className="card-desc">{t("appearance.uiTheme.desc")}</p>
                </div>
                <Segmented<UiTheme>
                  value={config.settings.uiTheme}
                  ariaLabel={t("appearance.uiTheme")}
                  onChange={(v) => updateSettings({ uiTheme: v })}
                  options={[
                    { value: "system", label: t("common.system") },
                    { value: "light", label: t("common.light"), icon: <SunIcon size={13} /> },
                    { value: "dark", label: t("common.dark"), icon: <MoonIcon size={13} /> },
                  ]}
                />
              </div>
            </div>

            <SettingRow title={t("appearance.accent")}>
              <ColorField
                value={config.settings.accentColor}
                ariaLabel={t("appearance.accent")}
                onChange={(v) => updateSettings({ accentColor: v })}
              />
            </SettingRow>

            <SettingRow
              title={t("appearance.animations")}
              desc={t("appearance.animations.desc")}
            >
              <Toggle
                checked={config.settings.animations}
                label={t("appearance.animations")}
                onChange={(v) => updateSettings({ animations: v })}
              />
            </SettingRow>

            <SettingRow
              title={t("appearance.transparency")}
              desc={t("appearance.transparency.desc")}
            >
              <Toggle
                checked={config.settings.transparency}
                label={t("appearance.transparency")}
                onChange={(v) => updateSettings({ transparency: v })}
              />
            </SettingRow>
            {!config.settings.transparency && (
              <p className="muted" style={{ marginTop: 4 }}>
                {t("appearance.transparency.desc")}
              </p>
            )}
          </div>
        )}

        {section === "advanced" && (
          <AdvancedSection
            state={state}
            onConfigChange={onConfigChange}
            onError={onError}
          />
        )}
      </div>
    </div>
  );
}

function CurrentThemeBanner({ state }: { state: AppStateResponse }) {
  const { t } = useI18n();
  const isLight = state.current.appModeLight;
  return (
    <div className="current-theme-banner">
      <span className={`status-dot ${isLight ? "light" : "dark"}`} style={{ width: 12, height: 12 }} />
      <div>
        <div className="label">{t("onboarding.s2.title")}</div>
        <div className="value">
          {isLight ? t("common.light") : t("common.dark")}
          {isLight ? " · Day" : " · Night"}
        </div>
      </div>
    </div>
  );
}

function AdvancedSection({
  state,
  onConfigChange,
  onError,
}: {
  state: AppStateResponse;
  onConfigChange: (config: AppConfig) => void;
  onError: (message: string) => void;
}) {
  const { t } = useI18n();
  const [updateMsg, setUpdateMsg] = useState<string | null>(null);
  const [configPath, setConfigPath] = useState<string>("");

  useEffect(() => {
    void api.configPath().then(setConfigPath);
  }, []);

  const copyDiagnostics = async () => {
    const info = await api.getDiagnostics();
    const ok = await copyText(info);
    if (ok) setUpdateMsg(t("common.copied"));
  };

  const checkUpdates = async () => {
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update) {
        setUpdateMsg(`${t("advanced.updateAvailable")}: ${update.version}`);
      } else {
        setUpdateMsg(t("advanced.upToDate"));
      }
    } catch (e) {
      setUpdateMsg(`${t("advanced.updateFailed")}: ${(e as Error).message ?? e}`);
    }
  };

  const restoreDefaults = async () => {
    try {
      const config = await api.resetConfig();
      onConfigChange(config);
    } catch (e) {
      onError((e as { message?: string }).message ?? String(e));
    }
  };

  return (
    <div className="anim">
      <h2 className="page-title">{t("nav.advanced")}</h2>
      <p className="page-subtitle">{state.osDisplay}</p>

      <div className="card">
        <p className="card-title">{t("advanced.apiInfo")}</p>
        <p className="card-desc">{t("advanced.apiInfo.desc")}</p>
      </div>

      <p className="card-section-title">{t("advanced.registryKeys")}</p>
      <div className="code-box">
        {`HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize
  AppsUseLightTheme     (REG_DWORD)
  SystemUsesLightTheme  (REG_DWORD)
  EnableTransparency    (REG_DWORD)
HKCU\\Software\\Microsoft\\Windows\\DWM
  AccentColor           (REG_DWORD, ABGR)
  ColorPrevalence       (REG_DWORD)
HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run
  Windows Theme Switcher (REG_SZ, autostart)
SystemParametersInfoW(SPI_SETDESKWALLPAPER) + WM_SETTINGCHANGE broadcast`}
      </div>

      <p className="card-section-title">{t("advanced.diagnostics")}</p>
      <div className="card">
        <div className="card-row">
          <div className="card-row-text">
            <p className="card-desc">{t("advanced.diagnostics.desc")}</p>
          </div>
          <button className="btn" onClick={copyDiagnostics}>
            {t("advanced.copyDiagnostics")}
          </button>
        </div>
      </div>

      <div className="card">
        <div className="card-row">
          <div className="card-row-text">
            <p className="card-title">{t("advanced.checkUpdates")}</p>
            {updateMsg && <p className="card-desc">{updateMsg}</p>}
          </div>
          <button className="btn" onClick={checkUpdates}>
            {t("advanced.checkUpdates")}
          </button>
        </div>
      </div>

      <p className="card-section-title">{t("advanced.restoreDefaults")}</p>
      <div className="card">
        <div className="card-row">
          <div className="card-row-text">
            <p className="card-desc">{t("advanced.restoreDefaults.desc")}</p>
          </div>
          <button className="btn danger" onClick={restoreDefaults}>
            {t("advanced.restoreDefaults")}
          </button>
        </div>
      </div>

      <p className="card-section-title">{t("advanced.configPath")}</p>
      <div className="code-box">{configPath || "…"}</div>

      <p className="card-section-title">{t("advanced.logPath")}</p>
      <div className="code-box">{state.logPath}</div>
    </div>
  );
}
