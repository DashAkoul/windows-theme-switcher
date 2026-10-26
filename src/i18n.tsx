// Minimal i18n with English and Persian (RTL) support.

import { createContext, useCallback, useContext, useMemo, type ReactNode } from "react";
import type { Language } from "./types";

type Dict = Record<string, [string, string]>; // [en, fa]

const DICT: Dict = {
  "app.name": ["Windows Theme Switcher", "سوییچر تم ویندوز"],
  "app.tagline": ["One-click Light / Dark theme switching", "تغییر یک‌کلیکی تم روشن و تیره"],

  // Common
  "common.save": ["Save", "ذخیره"],
  "common.saved": ["Saved", "ذخیره شد"],
  "common.cancel": ["Cancel", "انصراف"],
  "common.apply": ["Apply", "اعمال"],
  "common.detect": ["Detect current", "تشخیص وضعیت فعلی"],
  "common.browse": ["Browse…", "مرور…"],
  "common.dontChange": ["Don't change", "تغییر نده"],
  "common.on": ["On", "فعال"],
  "common.off": ["Off", "غیرفعال"],
  "common.light": ["Light", "روشن"],
  "common.dark": ["Dark", "تیره"],
  "common.copy": ["Copy", "کپی"],
  "common.copied": ["Copied to clipboard", "در کلیپ‌بورد کپی شد"],
  "common.close": ["Close", "بستن"],
  "common.reset": ["Reset", "بازنشانی"],
  "common.system": ["System", "سیستم"],

  // Onboarding
  "onboarding.skip": ["Skip", "رد کردن"],
  "onboarding.next": ["Next", "بعدی"],
  "onboarding.back": ["Back", "قبلی"],
  "onboarding.done": ["Done", "پایان"],
  "onboarding.openSettings": ["Open Settings", "بازکردن تنظیمات"],
  "onboarding.s1.title": ["Switch Windows Theme instantly.", "تم ویندوز را فوری عوض کن."],
  "onboarding.s1.desc": [
    "No more Settings → Personalization → Colors. One click on the tray icon switches your entire Windows theme.",
    "دیگر نه Settings → Personalization → Colors. فقط یک کلیک روی آیکون کنار ساعت، کل تم ویندوز را عوض می‌کند.",
  ],
  "onboarding.s2.title": ["Click the tray icon to switch between Day and Night.", "روی آیکون کنار ساعت بزن تا بین روز و شب جابه‌جا شوی."],
  "onboarding.s2.desc": [
    "The icon shows what happens next: a moon while Windows is Light, a sun while it is Dark. Right-click for more options.",
    "آیکون نشان می‌دهد کلیک چه می‌کند: ماه وقتی ویندوز روشن است، خورشید وقتی تیره است. راست‌کلیک برای گزینه‌های بیشتر.",
  ],
  "onboarding.s3.title": ["Customize both themes in Settings.", "هر دو تم را در تنظیمات شخصی‌سازی کن."],
  "onboarding.s3.desc": [
    "Define exactly what Light and Dark mean: Windows mode, app mode, transparency, accent color and wallpaper.",
    "دقیقاً تعیین کن که روشن و تیره یعنی چه: حالت ویندوز، حالت اپ‌ها، شفافیت، رنگ تأکیدی و تصویر پس‌زمینه.",
  ],

  // Navigation
  "nav.general": ["General", "عمومی"],
  "nav.light": ["Light (Day)", "روشن (روز)"],
  "nav.dark": ["Dark (Night)", "تیره (شب)"],
  "nav.appearance": ["Appearance", "ظاهر"],
  "nav.advanced": ["Advanced", "پیشرفته"],

  // General
  "general.startWithWindows": ["Start with Windows", "اجرا با ویندوز"],
  "general.startWithWindows.desc": ["Launch automatically when you sign in, straight to the tray.", "اجرای خودکار هنگام ورود، مستقیم در کنار ساعت."],
  "general.launchMinimized": ["Launch minimized", "اجرای مینیمایز"],
  "general.launchMinimized.desc": ["Start hidden in the tray without showing the window.", "شروع پنهان در تری بدون نمایش پنجره."],
  "general.checkUpdates": ["Check for updates", "بررسی به‌روزرسانی"],
  "general.checkUpdates.desc": ["Look for new versions in the background.", "جستجوی نسخه جدید در پس‌زمینه."],
  "general.language": ["Language", "زبان"],

  // Profile editor
  "profile.name": ["Profile name", "نام پروفایل"],
  "profile.windowsMode": ["Windows Mode", "حالت ویندوز"],
  "profile.windowsMode.desc": ["Taskbar, Start menu, widgets and system surfaces.", "تسک‌بار، منوی Start، ویجت‌ها و سطوح سیستم."],
  "profile.appMode": ["App Mode", "حالت اپلیکیشن‌ها"],
  "profile.appMode.desc": ["Default theme for Windows applications.", "تم پیش‌فرض اپلیکیشن‌های ویندوز."],
  "profile.transparency": ["Transparency", "شفافیت"],
  "profile.transparency.desc": ["Window and taskbar transparency effects.", "افکت‌های شفافیت پنجره و تسک‌بار."],
  "profile.accentColor": ["Accent Color", "رنگ تأکیدی"],
  "profile.accentColor.desc": ["Highlight color for buttons and controls.", "رنگ تأکیدی دکمه‌ها و کنترل‌ها."],
  "profile.wallpaper": ["Wallpaper", "تصویر پس‌زمینه"],
  "profile.wallpaper.desc": ["Desktop background image.", "تصویر پس‌زمینه دسکتاپ."],
  "profile.applyNow": ["Apply now", "اکنون اعمال کن"],
  "profile.applied": ["Profile applied to Windows", "پروفایل روی ویندوز اعمال شد"],
  "profile.detected": ["Current Windows settings detected", "تنظیمات فعلی ویندوز تشخیص داده شد"],

  // Preview
  "preview.title": ["Theme Preview", "پیش‌نمایش تم"],
  "preview.hint": ["Live preview of how this profile looks.", "پیش‌نمایش زنده از ظاهر این پروفایل."],

  // Appearance
  "appearance.uiTheme": ["UI Theme", "تم رابط کاربری"],
  "appearance.uiTheme.desc": ["Theme of the app window itself.", "تم پنجره خود برنامه."],
  "appearance.accent": ["Accent Color", "رنگ تأکیدی"],
  "appearance.animations": ["Animations", "انیمیشن‌ها"],
  "appearance.animations.desc": ["Animated transitions between views.", "انتقال‌های انیمیشنی بین صفحات."],
  "appearance.transparency": ["Window Transparency", "شفافیت پنجره"],
  "appearance.transparency.desc": ["Mica (Windows 11) or Acrylic (Windows 10) background.", "پس‌زمینه Mica (ویندوز ۱۱) یا Acrylic (ویندوز ۱۰)."],

  // Advanced
  "advanced.apiInfo": ["Windows API integration", "یکپارچه‌سازی با Windows API"],
  "advanced.apiInfo.desc": [
    "The app writes the same registry values the Settings app writes and broadcasts WM_SETTINGCHANGE. No admin rights required.",
    "برنامه همان مقادیر رجیستری را می‌نویسد که اپ Settings می‌نویسد و WM_SETTINGCHANGE را broadcast می‌کند. نیازی به دسترسی ادمین نیست.",
  ],
  "advanced.registryKeys": ["Registry keys in use", "کلیدهای رجیستری مورد استفاده"],
  "advanced.restoreDefaults": ["Restore defaults", "بازگردانی پیش‌فرض‌ها"],
  "advanced.restoreDefaults.desc": ["Reset all settings and profiles to their defaults.", "بازنشانی تمام تنظیمات و پروفایل‌ها."],
  "advanced.diagnostics": ["Diagnostics", "عیب‌یابی"],
  "advanced.diagnostics.desc": ["Copy system and app state for troubleshooting.", "کپی وضعیت سیستم و برنامه برای رفع اشکال."],
  "advanced.checkUpdates": ["Check for updates", "بررسی به‌روزرسانی"],
  "advanced.upToDate": ["You are running the latest version.", "آخرین نسخه را اجرا می‌کنید."],
  "advanced.updateAvailable": ["Update available", "به‌روزرسانی موجود است"],
  "advanced.updateFailed": ["Update check failed", "بررسی به‌روزرسانی ناموفق بود"],
  "advanced.configPath": ["Configuration file", "فایل پیکربندی"],
  "advanced.logPath": ["Log file", "فایل لاگ"],

  // About
  "about.desc": [
    "A lightweight tray utility that switches Windows between fully customizable Light and Dark themes with a single click.",
    "یک ابزار سبک تری که ویندوز را با یک کلیک بین دو تم کاملاً قابل‌شخصی‌سازی روشن و تیره جابه‌جا می‌کند.",
  ],
  "about.version": ["Version", "نسخه"],
  "about.github": ["Source on GitHub", "سورس در گیت‌هاب"],

  // Errors
  "error.title": ["Unable to change Windows theme.", "تغییر تم ویندوز ناموفق بود."],
  "error.retry": ["Retry", "تلاش مجدد"],
  "error.openSettings": ["Open Windows Settings", "باز کردن تنظیمات ویندوز"],
};

export type Translate = (key: string) => string;

const LanguageContext = createContext<{ lang: Language; t: Translate }>({
  lang: "en",
  t: (key) => key,
});

export function LanguageProvider({ lang, children }: { lang: Language; children: ReactNode }) {
  // NOTE: `lang` is used directly (not copied into state) so switching the
  // language in Settings re-renders every string immediately.
  const t: Translate = useCallback(
    (key) => {
      const entry = DICT[key];
      if (!entry) return key;
      return lang === "fa" ? entry[1] : entry[0];
    },
    [lang]
  );
  const value = useMemo(() => ({ lang, t }), [lang, t]);
  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>;
}

export function useI18n() {
  return useContext(LanguageContext);
}

/** Standalone translator for non-React code. */
export function translate(lang: Language, key: string): string {
  const entry = DICT[key];
  if (!entry) return key;
  return lang === "fa" ? entry[1] : entry[0];
}
