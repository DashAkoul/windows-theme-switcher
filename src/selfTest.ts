/// <reference types="vite/client" />

// Temporary UI self-test: reports layout/CSS diagnostics to the Rust log so
// the rendered geometry can be inspected without DevTools.

import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./api";

function rectOf(sel: string): string {
  const el = document.querySelector(sel);
  if (!el) return "missing";
  const r = el.getBoundingClientRect();
  return `x=${Math.round(r.x)} y=${Math.round(r.y)} w=${Math.round(r.width)} h=${Math.round(r.height)}`;
}

export function reportUiDiagnostics(tag: string): void {
  // Only in development builds - keeps release logs clean.
  if (!import.meta.env.DEV || !isTauri()) return;
  const root = document.getElementById("root");
  const bodyStyle = getComputedStyle(document.body);
  const shell = document.querySelector(".app-shell");
  const shellStyle = shell ? getComputedStyle(shell) : null;
  const info = {
    tag,
    viewport: `${window.innerWidth}x${window.innerHeight}@${window.devicePixelRatio}x`,
    theme: document.documentElement.dataset.theme ?? "none",
    dir: document.documentElement.dir || "ltr",
    lang: document.documentElement.lang || "en",
    styleSheets: document.styleSheets.length,
    bodyBg: bodyStyle.backgroundColor,
    bodyColor: bodyStyle.color,
    shellDisplay: shellStyle?.display ?? "missing",
    shellBg: shellStyle?.backgroundColor ?? "missing",
    rootChildren: root?.children.length ?? -1,
    rootScroll: root ? `${root.scrollWidth}x${root.scrollHeight}` : "missing",
    onboarding: rectOf(".onboarding"),
    icon: rectOf(".onboarding-icon"),
    title: rectOf(".onboarding-title"),
    desc: rectOf(".onboarding-desc"),
    actions: rectOf(".onboarding-actions"),
  };
  void invoke("log_debug", { message: JSON.stringify(info) }).catch(() => {});
}
