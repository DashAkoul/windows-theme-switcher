// Custom window title bar (the window uses `decorations: false`).
// Provides minimize / maximize / close buttons and a drag region.

import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function TitleBar({ title }: { title: string }) {
  const [maximized, setMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    appWindow
      .isMaximized()
      .then(setMaximized)
      .catch(() => {});
    // Refresh the state when the window is resized.
    const off = appWindow.onResized(() => {
      appWindow
        .isMaximized()
        .then(setMaximized)
        .catch(() => {});
    });
    off.then((f) => (unlisten = f));
    return () => unlisten?.();
  }, [appWindow]);

  return (
    <div className="titlebar">
      <div className="titlebar-drag" data-tauri-drag-region>
        <img className="titlebar-icon" src="/app-icon.svg" alt="" draggable={false} />
        <span className="titlebar-title">{title}</span>
      </div>
      <div className="titlebar-controls">
        <button
          className="titlebar-btn"
          title="Minimize"
          onClick={() => appWindow.minimize()}
        >
          <svg viewBox="0 0 12 12">
            <line x1="1" y1="10.5" x2="11" y2="10.5" stroke="currentColor" strokeWidth="1.2" />
          </svg>
        </button>
        <button
          className="titlebar-btn"
          title={maximized ? "Restore" : "Maximize"}
          onClick={() => appWindow.toggleMaximize()}
        >
          {maximized ? (
            <svg viewBox="0 0 12 12">
              <rect x="1" y="3" width="7" height="7" fill="none" stroke="currentColor" strokeWidth="1.2" />
              <path d="M3.5 3.5V1.5H10.5V8.5H8.5" fill="none" stroke="currentColor" strokeWidth="1.2" />
            </svg>
          ) : (
            <svg viewBox="0 0 12 12">
              <rect x="1.5" y="1.5" width="9" height="9" fill="none" stroke="currentColor" strokeWidth="1.2" />
            </svg>
          )}
        </button>
        <button
          className="titlebar-btn close"
          title="Close"
          onClick={() => appWindow.close()}
        >
          <svg viewBox="0 0 12 12">
            <line x1="1.5" y1="1.5" x2="10.5" y2="10.5" stroke="currentColor" strokeWidth="1.2" />
            <line x1="10.5" y1="1.5" x2="1.5" y2="10.5" stroke="currentColor" strokeWidth="1.2" />
          </svg>
        </button>
      </div>
    </div>
  );
}
