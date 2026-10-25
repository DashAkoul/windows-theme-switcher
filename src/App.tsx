// App root: loads state, applies the UI theme, wires events and routes
// between Onboarding, Settings and About.

import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TitleBar } from "./components/TitleBar";
import { ErrorToast, type ToastData } from "./components/Toast";
import { Onboarding, hideToTray } from "./views/Onboarding";
import { SettingsView } from "./views/SettingsView";
import { AboutView } from "./views/AboutView";
import { LanguageProvider, useI18n } from "./i18n";
import { api, onSystemThemeChanged, onThemeApplied, onThemeError, onTrayNavigate } from "./api";
import type { AppConfig, AppStateResponse, Language, ThemeState } from "./types";

type View = "onboarding" | "settings" | "about";

function AppInner() {
  const { t } = useI18n();
  const [state, setState] = useState<AppStateResponse | null>(null);
  const [view, setView] = useState<View>("onboarding");
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
  // Initial load + event wiring
  // ------------------------------------------------------------------
  useEffect(() => {
    let disposed = false;
    const unlisteners: (() => void)[] = [];

    (async () => {
      try {
        const s = await api.getState();
        if (disposed) return;
        setState(s);
        setView(s.config.settings.onboardingCompleted ? "settings" : "onboarding");

        // Consume the pending route set by the tray menu.
        if (s.pendingRoute === "about") {
          setView("about");
        } else if (s.pendingRoute === "settings") {
          setView("settings");
        }
        if (s.pendingRoute) {
          await api.clearPendingRoute();
        }
      } catch (e) {
        pushToast((e as { message?: string }).message ?? String(e));
      }
    })();

    void onSystemThemeChanged((current: ThemeState) => {
      setState((prev) => (prev ? { ...prev, current } : prev));
    }).then((off) => unlisteners.push(off));

    void onThemeApplied((current: ThemeState) => {
      setState((prev) => (prev ? { ...prev, current } : prev));
    }).then((off) => unlisteners.push(off));

    void onThemeError((payload) => {
      pushToast(payload.message);
      lastAction.current = () => {
        void api.toggleTheme().catch((e) => pushToast((e as { message?: string }).message ?? String(e)));
      };
    }).then((off) => unlisteners.push(off));

    void onTrayNavigate((route) => {
      if (route === "about") setView("about");
      else setView("settings");
    }).then((off) => unlisteners.push(off));

    return () => {
      disposed = true;
      unlisteners.forEach((off) => off());
    };
  }, [pushToast]);

  // ------------------------------------------------------------------
  // Theme + language application
  // ------------------------------------------------------------------
  useEffect(() => {
    if (!state) return;
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
  }, [state]);

  // ------------------------------------------------------------------
  // Handlers
  // ------------------------------------------------------------------
  const finishOnboarding = useCallback(
    async (openSettings: boolean) => {
      if (!state) return;
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
    [state, pushToast]
  );

  const onConfigChange = useCallback((config: AppConfig) => {
    setState((prev) => (prev ? { ...prev, config } : prev));
  }, []);

  const onError = useCallback(
    (message: string) => {
      pushToast(message);
      lastAction.current = () => {
        void api.toggleTheme().catch(() => {});
      };
    },
    [pushToast]
  );

  if (!state) {
    return (
      <div className="app-shell">
        <TitleBar title={t("app.name")} />
        <div style={{ flex: 1, display: "grid", placeItems: "center" }}>
          <span className="muted">…</span>
        </div>
      </div>
    );
  }

  const langKey: Language = state.config.settings.language;

  return (
    <LanguageProvider lang={langKey}>
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
    </LanguageProvider>
  );
}

export default function App() {
  // The language is only known after the first state load; render the
  // provider at the inner level and keep a stable outer shell.
  const [lang, setLang] = useState<Language>("en");
  useEffect(() => {
    void getCurrentWindow(); // keep the window handle warm
    void api
      .getState()
      .then((s) => setLang(s.config.settings.language))
      .catch(() => {});
  }, []);
  return (
    <LanguageProvider lang={lang}>
      <AppInner />
    </LanguageProvider>
  );
}
