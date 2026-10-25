// Error toast with the actions promised by the spec:
// Retry / Open Windows Settings / Copy diagnostics.

import { useI18n } from "../i18n";
import { api, copyText } from "../api";

export interface ToastData {
  id: number;
  message: string;
}

interface Props {
  toast: ToastData;
  onDismiss: (id: number) => void;
  onRetry: () => void;
}

export function ErrorToast({ toast, onDismiss, onRetry }: Props) {
  const { t } = useI18n();

  const copyDiagnostics = async () => {
    const info = await api.getDiagnostics();
    await copyText(info);
  };

  return (
    <div className="toast" role="alert">
      <p className="toast-title">{t("error.title")}</p>
      <p className="toast-message">{toast.message}</p>
      <div className="toast-actions">
        <button className="btn primary" onClick={onRetry}>
          {t("error.retry")}
        </button>
        <button
          className="btn"
          onClick={() => {
            void api.openWindowsSettings();
          }}
        >
          {t("error.openSettings")}
        </button>
        <button
          className="btn"
          onClick={() => {
            void copyDiagnostics();
          }}
        >
          {t("advanced.copyDiagnostics")}
        </button>
        <button className="btn" onClick={() => onDismiss(toast.id)}>
          {t("common.close")}
        </button>
      </div>
    </div>
  );
}
