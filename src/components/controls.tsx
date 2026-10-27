// Shared controls for the panel (icons + a small toggle).

/**
 * Title bar of the popup: drag region, app icon, title and the close button.
 *
 * Shared so the error screen has the same chrome as the normal panel - it used
 * to render without a header, which left the user with a popup that had no ✕
 * and could only be dismissed by clicking outside.
 */
export function PanelHeader({ onClose }: { onClose: () => void }) {
  return (
    <header className="panel-head" data-tauri-drag-region>
      <img className="panel-logo" src="/app-icon.svg" alt="" draggable={false} />
      <span className="panel-title" data-tauri-drag-region>
        Windows Theme Switcher
      </span>
      <button className="panel-close" title="Close" onClick={onClose}>
        <svg viewBox="0 0 12 12" aria-hidden="true">
          <line x1="1.5" y1="1.5" x2="10.5" y2="10.5" stroke="currentColor" strokeWidth="1.3" />
          <line x1="10.5" y1="1.5" x2="1.5" y2="10.5" stroke="currentColor" strokeWidth="1.3" />
        </svg>
      </button>
    </header>
  );
}

export function Toggle({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className={`toggle${checked ? " on" : ""}`}
      onClick={() => onChange(!checked)}
    />
  );
}

export function SunIcon({ size = 16 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="4.5" fill="currentColor" stroke="none" />
      <line x1="12" y1="1.5" x2="12" y2="4" />
      <line x1="12" y1="20" x2="12" y2="22.5" />
      <line x1="1.5" y1="12" x2="4" y2="12" />
      <line x1="20" y1="12" x2="22.5" y2="12" />
      <line x1="4.6" y1="4.6" x2="6.3" y2="6.3" />
      <line x1="17.7" y1="17.7" x2="19.4" y2="19.4" />
      <line x1="4.6" y1="19.4" x2="6.3" y2="17.7" />
      <line x1="17.7" y1="6.3" x2="19.4" y2="4.6" />
    </svg>
  );
}

export function MoonIcon({ size = 16 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path
        d="M20.5 14.5A8.5 8.5 0 1 1 9.5 3.5a7 7 0 0 0 11 11z"
        fill="currentColor"
        stroke="none"
      />
    </svg>
  );
}
