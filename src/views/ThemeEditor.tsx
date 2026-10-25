// Theme editor for one profile (Light or Dark) with a live preview.
// Changes are kept in a local draft and persisted with the Save button;
// "Apply now" applies the profile to Windows immediately.

import { useState } from "react";
import { useI18n } from "../i18n";
import { api, pickWallpaper } from "../api";
import { ColorField, Segmented, SettingRow, SunIcon, MoonIcon } from "../components/controls";
import { ThemePreview } from "../components/ThemePreview";
import type { ThemeMode, ThemeProfile } from "../types";

interface Props {
  profileId: string;
  profile: ThemeProfile;
  onSave: (profile: ThemeProfile) => void;
  onApplied: () => void;
  onError: (message: string) => void;
}

export function ThemeEditor({ profileId, profile, onSave, onApplied, onError }: Props) {
  const { t } = useI18n();
  const [draft, setDraft] = useState<ThemeProfile>(profile);
  const [dirty, setDirty] = useState(false);
  const [status, setStatus] = useState<string | null>(null);

  const update = <K extends keyof ThemeProfile>(key: K, value: ThemeProfile[K]) => {
    setDraft((d) => ({ ...d, [key]: value }));
    setDirty(true);
    setStatus(null);
  };

  const save = () => {
    onSave(draft);
    setDirty(false);
    setStatus(t("common.saved"));
  };

  const apply = async () => {
    try {
      if (dirty) onSave(draft);
      await api.applyProfile(profileId);
      setDirty(false);
      setStatus(t("profile.applied"));
      onApplied();
    } catch (e) {
      onError((e as { message?: string }).message ?? String(e));
    }
  };

  const detect = async () => {
    try {
      const detected = await api.detectCurrentProfile();
      setDraft({ ...detected, name: draft.name });
      setDirty(true);
      setStatus(t("profile.detected"));
    } catch (e) {
      onError((e as { message?: string }).message ?? String(e));
    }
  };

  const browse = async () => {
    const path = await pickWallpaper();
    if (path) update("wallpaper", path);
  };

  return (
    <div>
      <div className="editor-layout">
        <div>
          <div className="card">
            <div className="card-row">
              <div className="card-row-text">
                <p className="card-title">{t("profile.name")}</p>
              </div>
              <input
                className="text-input"
                style={{ maxWidth: 220 }}
                value={draft.name}
                onChange={(e) => update("name", e.target.value)}
              />
            </div>
          </div>

          <div className="card">
            <div className="card-row">
              <div className="card-row-text">
                <p className="card-title">{t("profile.windowsMode")}</p>
                <p className="card-desc">{t("profile.windowsMode.desc")}</p>
              </div>
              <Segmented<ThemeMode>
                value={draft.windowsMode}
                ariaLabel={t("profile.windowsMode")}
                onChange={(v) => update("windowsMode", v)}
                options={[
                  { value: "light", label: t("common.light"), icon: <SunIcon size={13} /> },
                  { value: "dark", label: t("common.dark"), icon: <MoonIcon size={13} /> },
                ]}
              />
            </div>
          </div>

          <div className="card">
            <div className="card-row">
              <div className="card-row-text">
                <p className="card-title">{t("profile.appMode")}</p>
                <p className="card-desc">{t("profile.appMode.desc")}</p>
              </div>
              <Segmented<ThemeMode>
                value={draft.appMode}
                ariaLabel={t("profile.appMode")}
                onChange={(v) => update("appMode", v)}
                options={[
                  { value: "light", label: t("common.light"), icon: <SunIcon size={13} /> },
                  { value: "dark", label: t("common.dark"), icon: <MoonIcon size={13} /> },
                ]}
              />
            </div>
          </div>

          <SettingRow title={t("profile.transparency")} desc={t("profile.transparency.desc")}>
            <button
              type="button"
              role="switch"
              aria-checked={draft.transparency}
              aria-label={t("profile.transparency")}
              className={`toggle${draft.transparency ? " on" : ""}`}
              onClick={() => update("transparency", !draft.transparency)}
            />
          </SettingRow>

          <div className="card">
            <div className="card-row">
              <div className="card-row-text">
                <p className="card-title">{t("profile.accentColor")}</p>
                <p className="card-desc">{t("profile.accentColor.desc")}</p>
              </div>
              <div className="btn-row">
                {draft.accentColor ? (
                  <>
                    <ColorField
                      value={draft.accentColor}
                      ariaLabel={t("profile.accentColor")}
                      onChange={(v) => update("accentColor", v)}
                    />
                    <button className="btn" onClick={() => update("accentColor", null)}>
                      {t("common.dontChange")}
                    </button>
                  </>
                ) : (
                  <button
                    className="btn"
                    onClick={() => update("accentColor", "#0078D4")}
                  >
                    {t("common.dontChange")}
                  </button>
                )}
              </div>
            </div>
          </div>

          <div className="card">
            <div className="card-row">
              <div className="card-row-text">
                <p className="card-title">{t("profile.wallpaper")}</p>
                <p className="card-desc">{t("profile.wallpaper.desc")}</p>
                {draft.wallpaper && (
                  <p className="card-desc" style={{ marginTop: 4, wordBreak: "break-all" }}>
                    {draft.wallpaper}
                  </p>
                )}
              </div>
              <div className="btn-row">
                <button className="btn" onClick={browse}>
                  {t("common.browse")}
                </button>
                {draft.wallpaper && (
                  <button className="btn" onClick={() => update("wallpaper", null)}>
                    {t("common.dontChange")}
                  </button>
                )}
              </div>
            </div>
          </div>

          <div className="btn-row mt-16">
            <button className="btn primary" onClick={save} disabled={!dirty}>
              {t("common.save")}
            </button>
            <button className="btn" onClick={apply}>
              {t("profile.applyNow")}
            </button>
            <button className="btn" onClick={detect}>
              {t("common.detect")}
            </button>
            {status && <span className="muted" style={{ alignSelf: "center" }}>{status}</span>}
          </div>
        </div>

        <ThemePreview profile={draft} />
      </div>
    </div>
  );
}
