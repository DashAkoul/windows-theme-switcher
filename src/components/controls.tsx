// Reusable form controls styled to match Windows 11.

import { useEffect, useState, type ReactNode } from "react";

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

export interface SegmentedOption<T extends string> {
  value: T;
  label: string;
  icon?: ReactNode;
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
}: {
  value: T;
  options: SegmentedOption<T>[];
  onChange: (v: T) => void;
  ariaLabel?: string;
}) {
  return (
    <div className="segmented" role="radiogroup" aria-label={ariaLabel}>
      {options.map((opt) => (
        <button
          key={opt.value}
          type="button"
          role="radio"
          aria-checked={value === opt.value}
          className={value === opt.value ? "active" : ""}
          onClick={() => onChange(opt.value)}
        >
          {opt.icon}
          {opt.label}
        </button>
      ))}
    </div>
  );
}

export function SettingRow({
  title,
  desc,
  children,
}: {
  title: string;
  desc?: string;
  children: ReactNode;
}) {
  return (
    <div className="card">
      <div className="card-row">
        <div className="card-row-text">
          <p className="card-title">{title}</p>
          {desc && <p className="card-desc">{desc}</p>}
        </div>
        {children}
      </div>
    </div>
  );
}

export function ColorField({
  value,
  onChange,
  ariaLabel,
}: {
  value: string;
  onChange: (v: string) => void;
  ariaLabel: string;
}) {
  // Keep a local draft so partial input ("#00") is never persisted: the
  // backend rejects invalid hex and every keystroke used to raise an error.
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  const commit = (next: string) => {
    setDraft(next);
    if (/^#[0-9a-fA-F]{6}$/.test(next)) {
      onChange(next.toUpperCase());
    }
  };

  const invalid = !/^#[0-9a-fA-F]{6}$/.test(draft);

  return (
    <div className="color-picker">
      <input
        className="color-swatch"
        type="color"
        value={/^#[0-9a-fA-F]{6}$/.test(draft) ? draft : value}
        aria-label={ariaLabel}
        onChange={(e) => commit(e.target.value.toUpperCase())}
      />
      <input
        className="text-input color-hex"
        type="text"
        value={draft}
        aria-label={`${ariaLabel} hex`}
        maxLength={7}
        style={invalid ? { borderColor: "var(--danger)" } : undefined}
        onChange={(e) => commit(e.target.value)}
      />
    </div>
  );
}

export function SunIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
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
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M20.5 14.5A8.5 8.5 0 1 1 9.5 3.5a7 7 0 0 0 11 11z" fill="currentColor" stroke="none" />
    </svg>
  );
}
