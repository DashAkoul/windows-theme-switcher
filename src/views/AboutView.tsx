// About view - shown when the tray "About" menu item is clicked.

import { useEffect, useState } from "react";
import { useI18n } from "../i18n";
import { api, isTauri } from "../api";

export function AboutView({ onBack }: { onBack: () => void }) {
  const { t } = useI18n();
  const [version, setVersion] = useState("1.0.1");

  useEffect(() => {
    if (!isTauri()) return;
    void import("@tauri-apps/api/app")
      .then((m) => m.getVersion())
      .then(setVersion)
      .catch(() => {});
  }, []);

  return (
    <div className="content">
      <div className="anim" style={{ maxWidth: 560 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 16, marginBottom: 20 }}>
          <img src="/app-icon.svg" width={64} height={64} alt="" draggable={false} />
          <div>
            <h2 className="page-title" style={{ margin: 0 }}>
              {t("app.name")}
            </h2>
            <p className="muted" style={{ margin: 0 }}>
              {t("app.tagline")}
            </p>
          </div>
        </div>

        <p style={{ color: "var(--text-secondary)", userSelect: "text" }}>{t("about.desc")}</p>

        <div className="card">
          <div className="card-row">
            <div className="card-row-text">
              <p className="card-title">{t("about.version")}</p>
            </div>
            <span className="muted">{version}</span>
          </div>
        </div>

        <div className="card">
          <div className="card-row">
            <div className="card-row-text">
              <p className="card-title">GNU GPL v3.0</p>
              <p className="card-desc">
                Free / copyleft software — you may use, study, share and modify
                it, provided derivative works stay under the same license.
              </p>
            </div>
            <button
              className="btn"
              onClick={() => {
                void api.openUrl("https://www.gnu.org/licenses/gpl-3.0.html");
              }}
            >
              GPL-3.0
            </button>
          </div>
        </div>

        <div className="btn-row mt-16">
          <button className="btn" onClick={onBack}>
            {t("common.close")}
          </button>
          <button
            className="btn"
            onClick={() => {
              void api.openUrl("https://github.com/DashAkoul/windows-theme-switcher");
            }}
          >
            {t("about.github")}
          </button>
        </div>
      </div>
    </div>
  );
}
