// The whole UI: a compact tray popup with the Light / Dark switch.

import { useCallback, useEffect, useState } from "react";
import { api, closePanel, errorMessage, isTauri, onThemeChanged } from "../api";
import { MoonIcon, SunIcon, Toggle } from "../components/controls";
import type { AppStateResponse } from "../types";
import { DARK_PROFILE_ID, LIGHT_PROFILE_ID } from "../types";

export function Panel({ initial }: { initial: AppStateResponse }) {
  const [state, setState] = useState<AppStateResponse>(initial);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [update, setUpdate] = useState<string | null>(null);
  const [updating, setUpdating] = useState(false);

  // Windows (or the tray) may change the theme while the panel is open.
  useEffect(() => {
    let off: (() => void) | undefined;
    void onThemeChanged((current) => {
      setState((prev) => ({ ...prev, current }));
    }).then((fn) => (off = fn));
    return () => off?.();
  }, []);

  // Silent update check (only shows a link when an update really exists).
  useEffect(() => {
    if (!isTauri() || !state.config.settings.checkUpdates) return;
    let cancelled = false;
    void (async () => {
      try {
        const { check } = await import("@tauri-apps/plugin-updater");
        const found = await check();
        if (!cancelled && found) setUpdate(found.version);
      } catch {
        // Offline or no endpoint configured: ignore.
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [state.config.settings.checkUpdates]);

  const installUpdate = useCallback(async () => {
    if (updating) return;
    setUpdating(true);
    setError(null);
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const found = await check();
      if (found) await found.downloadAndInstall();
      setUpdate(null);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setUpdating(false);
    }
  }, [updating]);

  const isLight = state.current.appModeLight;

  const apply = useCallback(
    async (id: string) => {
      if (busy) return;
      setBusy(id);
      setError(null);
      try {
        const current = await api.applyProfile(id);
        setState((prev) => ({ ...prev, current }));
      } catch (e) {
        setError(errorMessage(e));
      } finally {
        setBusy(null);
      }
    },
    [busy]
  );

  const setAutostart = useCallback(
    async (enabled: boolean) => {
      setError(null);
      try {
        const config = await api.saveSettings({
          ...state.config.settings,
          startWithWindows: enabled,
        });
        setState((prev) => ({ ...prev, config }));
      } catch (e) {
        setError(errorMessage(e));
      }
    },
    [state.config.settings]
  );

  return (
    <div className="panel">
      <header className="panel-head" data-tauri-drag-region>
        <img className="panel-logo" src="/app-icon.svg" alt="" draggable={false} />
        <span className="panel-title" data-tauri-drag-region>
          Windows Theme Switcher
        </span>
        <button className="panel-close" title="Close" onClick={closePanel}>
          <svg viewBox="0 0 12 12" aria-hidden="true">
            <line x1="1.5" y1="1.5" x2="10.5" y2="10.5" stroke="currentColor" strokeWidth="1.3" />
            <line x1="10.5" y1="1.5" x2="1.5" y2="10.5" stroke="currentColor" strokeWidth="1.3" />
          </svg>
        </button>
      </header>

      <div className="panel-body">
        <div className="mode-grid" role="radiogroup" aria-label="Windows theme">
          <button
            type="button"
            role="radio"
            aria-checked={isLight}
            className={`mode${isLight ? " active" : ""}`}
            disabled={busy !== null}
            onClick={() => apply(LIGHT_PROFILE_ID)}
          >
            <SunIcon size={22} />
            <span>Light</span>
          </button>
          <button
            type="button"
            role="radio"
            aria-checked={!isLight}
            className={`mode${!isLight ? " active" : ""}`}
            disabled={busy !== null}
            onClick={() => apply(DARK_PROFILE_ID)}
          >
            <MoonIcon size={22} />
            <span>Dark</span>
          </button>
        </div>

        <div className="row">
          <span className="row-label">Start with Windows</span>
          <Toggle
            checked={state.config.settings.startWithWindows}
            onChange={setAutostart}
            label="Start with Windows"
          />
        </div>

        <button className="link" onClick={() => void api.openWindowsSettings()}>
          Windows theme settings…
        </button>

        {error && <p className="error">{error}</p>}

        <footer className="panel-foot">
          {update ? (
            <button className="link" onClick={installUpdate} disabled={updating}>
              {updating ? "Installing update…" : `Update to v${update}`}
            </button>
          ) : (
            <span>v{state.version}</span>
          )}
        </footer>
      </div>
    </div>
  );
}
