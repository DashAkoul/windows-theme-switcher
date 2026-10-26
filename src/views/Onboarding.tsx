// Three-step first-run onboarding. On finish the app moves to the tray.

import { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "../i18n";
import { MoonIcon, SunIcon } from "../components/controls";
import { isTauri } from "../api";

interface Props {
  onFinish: (openSettings: boolean) => void;
}

const STEPS = [0, 1, 2] as const;

export function Onboarding({ onFinish }: Props) {
  const { t } = useI18n();
  const [step, setStep] = useState(0);
  const isLast = step === STEPS.length - 1;

  const finish = (openSettings: boolean) => {
    onFinish(openSettings);
  };

  return (
    <div className="onboarding anim">
      <div className="onboarding-icon">
        {step === 0 && <MoonIcon size={120} />}
        {step === 1 && <SunIcon size={120} />}
        {step === 2 && (
          <svg width="120" height="120" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        )}
      </div>

      <h1 className="onboarding-title">{t(`onboarding.s${step + 1}.title`)}</h1>
      <p className="onboarding-desc">{t(`onboarding.s${step + 1}.desc`)}</p>

      <div className="onboarding-dots">
        {STEPS.map((s) => (
          <button
            key={s}
            className={`onboarding-dot${s === step ? " active" : ""}`}
            aria-label={`Step ${s + 1}`}
            onClick={() => setStep(s)}
          />
        ))}
      </div>

      <div className="onboarding-actions">
        {step > 0 && (
          <button className="btn" onClick={() => setStep(step - 1)}>
            {t("onboarding.back")}
          </button>
        )}
        {step < STEPS.length - 1 && (
          <>
            <button className="btn" onClick={() => finish(false)}>
              {t("onboarding.skip")}
            </button>
            <button className="btn primary" onClick={() => setStep(step + 1)}>
              {t("onboarding.next")}
            </button>
          </>
        )}
        {isLast && (
          <>
            <button className="btn" onClick={() => finish(false)}>
              {t("onboarding.done")}
            </button>
            <button className="btn primary" onClick={() => finish(true)}>
              {t("onboarding.openSettings")}
            </button>
          </>
        )}
      </div>
    </div>
  );
}

/** Close the window (the app keeps running in the tray, memory is freed). */
export function hideToTray() {
  if (!isTauri()) return;
  void getCurrentWindow().close();
}
