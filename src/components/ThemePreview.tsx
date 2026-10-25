// Live preview of a theme profile: a mock application window rendered with
// the profile's colors, shown over the profile's wallpaper (if any).

import { useI18n } from "../i18n";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { ThemeProfile } from "../types";

interface Props {
  profile: ThemeProfile;
}

const LIGHT_COLORS = {
  windowBg: "rgba(243,243,243,0.92)",
  text: "#1a1a1a",
  titlebar: "rgba(250,250,250,0.9)",
  controlBg: "#ffffff",
  border: "rgba(0,0,0,0.08)",
};

const DARK_COLORS = {
  windowBg: "rgba(32,32,32,0.92)",
  text: "#ffffff",
  titlebar: "rgba(43,43,43,0.9)",
  controlBg: "#3a3a3a",
  border: "rgba(255,255,255,0.09)",
};

export function ThemePreview({ profile }: Props) {
  const { t } = useI18n();
  const dark = profile.appMode === "dark";
  const colors = dark ? DARK_COLORS : LIGHT_COLORS;
  const accent = profile.accentColor ?? (dark ? "#6b69d6" : "#0078d4");

  const desktopStyle: React.CSSProperties = {
    background: profile.wallpaper
      ? `url("${convertFileSrc(profile.wallpaper)}") center / cover no-repeat`
      : "linear-gradient(135deg, #5b7c99, #2c3e50)",
  };

  return (
    <div className="preview-panel">
      <p className="preview-label">{t("preview.title")}</p>
      <div className="preview-desktop" style={desktopStyle}>
        <div
          className={`preview-window${profile.transparency ? " transparency" : ""}`}
          style={{
            background: colors.windowBg,
            color: colors.text,
            borderColor: colors.border,
          }}
        >
          <div
            className="preview-titlebar"
            style={{ background: colors.titlebar, borderBottom: `1px solid ${colors.border}` }}
          >
            <span className="preview-dot" style={{ background: "#ff5f57" }} />
            <span className="preview-dot" style={{ background: "#febc2e" }} />
            <span className="preview-dot" style={{ background: "#28c840" }} />
            <span className="preview-title-text">Sample Application</span>
          </div>
          <div className="preview-body">
            <p className="preview-heading">Hello, Windows!</p>
            <p className="preview-text">
              This is how your apps will look with this theme applied.
            </p>
            <div className="preview-controls">
              <span
                className={`preview-toggle${profile.transparency ? " on" : ""}`}
                style={{ background: profile.transparency ? accent : colors.controlBg }}
              />
              <span className="preview-toggle-label">
                {profile.transparency ? t("common.on") : t("common.off")}
              </span>
            </div>
            <div className="preview-buttons">
              <button
                className="preview-btn primary"
                style={{ background: accent }}
                type="button"
              >
                Primary
              </button>
              <button className="preview-btn" style={{ borderColor: colors.border }} type="button">
                Cancel
              </button>
            </div>
          </div>
        </div>
      </div>
      <div className="preview-meta">
        <span className="preview-chip">
          {t("profile.windowsMode")}: {profile.windowsMode === "light" ? t("common.light") : t("common.dark")}
        </span>
        <span className="preview-chip">
          {t("profile.appMode")}: {profile.appMode === "light" ? t("common.light") : t("common.dark")}
        </span>
        <span className="preview-chip">
          {t("profile.transparency")}: {profile.transparency ? t("common.on") : t("common.off")}
        </span>
        {profile.accentColor && (
          <span className="preview-chip">
            <span
              className="status-dot"
              style={{ background: profile.accentColor, marginInlineEnd: 5 }}
            />
            {profile.accentColor}
          </span>
        )}
      </div>
    </div>
  );
}
