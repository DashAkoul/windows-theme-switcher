// App root: loads state once, applies the UI theme, wires events and routes
// between Onboarding, Settings and About.

import { useCallback, useEffect, useRef, useState } from "react";
import { TitleBar } from "./components/TitleBar";
import { ErrorToast, type ToastData } from "./components/Toast";
import { Onboarding, hideToTray } from "./views/Onboarding";
import { SettingsView } from "./views/SettingsView";
import { AboutView } from "./views/AboutView";
import { LanguageProvider, useI18n } from "./i18n";
import { api, onSystemThemeChanged, onThemeApplied, onThemeError, onTrayNavigate } from "./api";
import { reportUiDiagnostics } from "./selfTest";
import type { AppConfig, AppStateResponse, ThemeState } from "./types";

type View = "onboarding" | "settings" | "about";

export default function App() {
  const [state, setState] = useState<AppStateResponse | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);

  // ------------------------------------------------------------------
  // Initial load
  // ------------------------------------------------------------------
  useEffect(() => {
    let disposed = false;
    setLoadError(null);
    (async () => {
      try {
        const s = await api.getState();
        if (disposed) return;
        setState(s);

        // Consume the pending route set by the tray menu.
        if (s.pendingRoute) {
          await api.clearPendingRoute();
        }
      } catch (err) {
        // Never leave the user on a blank window: show an actionable error.
        console.error("[wts] state load failed", err);
        if (!disposed) {
          const msg = (err as { message?: string }).message ?? String(err);
          setLoadError(msg);
        }
      }
    })();
    return () => {
      disposed = true;
    };
  }, [attempt]);

  if (!state) {
    // Render a minimal shell until the state arrives (usually < 100ms).
    return (
      <LanguageProvider lang="en">
        <LoadingShell
          error={loadError}
          onRetry={() => setAttempt((n) => n + 1)}
        />
      </LanguageProvider>
    );
  }

  return (
    <LanguageProvider lang={state.config.settings.language}>
      <AppShell state={state} setState={setState} />
    </LanguageProvider>
  );
}

function LoadingShell({
  error,
  onRetry,
}: {
  error: string | null;
  onRetry: () => void;
}) {
  const { t } = useI18n();
  return (
    <div className="app-shell">
      <TitleBar title={t("app.name")} />
      <div style={{ flex: 1, display: "grid", placeItems: "center", padding: 24 }}>
        {error ? (
          <div style={{ maxWidth: 460, textAlign: "center" }}>
            <p style={{ fontWeight: 600, marginBottom: 6 }}>{t("error.title")}</p>
            <p className="muted" style={{ marginBottom: 14, wordBreak: "break-word" }}>
              {error}
            </p>
            <button className="btn primary" onClick={onRetry}>
              {t("error.retry")}
            </button>
          </div>
        ) : (
          <span className="muted">…</span>
        )}
      </div>
    </div>
  );
}

interface ShellProps {
  state: AppStateResponse;
  setState: React.Dispatch<React.SetStateAction<AppStateResponse | null>>;
}

function AppShell({ state, setState }: ShellProps) {
  const { t } = useI18n();
  const [view, setView] = useState<View>(() =>
    state.config.settings.onboardingCompleted ? "settings" : "onboarding"
  );
  const [toasts, setToasts] = useState<ToastData[]>([]);
  const toastId = useRef(0);
  const lastAction = useRef<(() => void) | null>(null);

  const pushToast = useCallback((message: string) => {
    const id = ++toastId.current;
    setToasts((list) => [...list, { id, message }]);
  }, []);

  const dismissToast = useCallback((id: number) => {
    setToasts((list) => list.filter((toast) => toast.id !== id));
  }, []);

  const retry = useCallback(() => {
    if (lastAction.current) {
      lastAction.current();
      setToasts([]);
    }
  }, []);

  // ------------------------------------------------------------------
  // Event wiring
  // ------------------------------------------------------------------
  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    void onSystemThemeChanged((current: ThemeState) => {
      setState((prev) => (prev ? { ...prev, current } : prev));
    }).then((off) => unlisteners.push(off));

    void onThemeApplied((current: ThemeState) => {
      setState((prev) => (prev ? { ...prev, current } : prev));
    }).then((off) => unlisteners.push(off));

    void onThemeError((payload) => {
      pushToast(payload.message);
      lastAction.current = () => {
        void api.toggleTheme().catch(() => {});
      };
    }).then((off) => unlisteners.push(off));

    void onTrayNavigate((route) => {
      if (route === "about") setView("about");
      else setView("settings");
    }).then((off) => unlisteners.push(off));

    return () => {
      unlisteners.forEach((off) => off());
    };
  }, [pushToast, setState]);

  // ------------------------------------------------------------------
  // Theme + language application
  // ------------------------------------------------------------------
  useEffect(() => {
    const settings = state.config.settings;
    const current = state.current;

    const effectiveDark =
      settings.uiTheme === "dark" ||
      (settings.uiTheme === "system" && !current.appModeLight);

    document.documentElement.dataset.theme = effectiveDark ? "dark" : "light";
    document.documentElement.style.setProperty("--accent", settings.accentColor);
    document.documentElement.lang = settings.language;
    document.documentElement.dir = settings.language === "fa" ? "rtl" : "ltr";
    document.documentElement.classList.toggle("no-anim", !settings.animations);

    // Dev-only layout diagnostics (no-op in release builds).
    const id = window.setTimeout(() => reportUiDiagnostics(`view=${view}`), 600);
    return () => window.clearTimeout(id);
  }, [state, view]);

  // ------------------------------------------------------------------
  // Handlers
  // ------------------------------------------------------------------
  const finishOnboarding = useCallback(
    async (openSettings: boolean) => {
      const config: AppConfig = {
        ...state.config,
        settings: { ...state.config.settings, onboardingCompleted: true },
      };
      try {
        const saved = await api.saveConfig(config);
        setState((prev) => (prev ? { ...prev, config: saved } : prev));
      } catch (e) {
        pushToast((e as { message?: string }).message ?? String(e));
        return;
      }
      if (openSettings) {
        setView("settings");
      } else {
        hideToTray();
      }
    },
    [state, pushToast, setState]
  );

  const onConfigChange = useCallback(
    (config: AppConfig) => {
      setState((prev) => (prev ? { ...prev, config } : prev));
    },
    [setState]
  );

  const onError = useCallback(
    (message: string) => {
      pushToast(message);
      lastAction.current = () => {
        void api.toggleTheme().catch(() => {});
      };
    },
    [pushToast]
  );

  return (
    <div className="app-shell">
      <TitleBar title={t("app.name")} />
      {view === "onboarding" && <Onboarding onFinish={finishOnboarding} />}
      {view === "settings" && (
        <SettingsView
          state={state}
          onConfigChange={onConfigChange}
          onError={onError}
          onCurrentChange={(current) =>
            setState((prev) => (prev ? { ...prev, current } : prev))
          }
        />
      )}
      {view === "about" && <AboutView onBack={() => setView("settings")} />}

      <div className="toast-container">
        {toasts.map((toast) => (
          <ErrorToast
            key={toast.id}
            toast={toast}
            onDismiss={dismissToast}
            onRetry={retry}
          />
        ))}
      </div>
    </div>
  );
}
