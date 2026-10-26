# Windows Theme Switcher

**سوییچر تم ویندوز** — یک ابزار سبک System Tray برای Windows 10/11 که با **یک کلیک** روی آیکون کنار ساعت، حالت ظاهری ویندوز را بین دو پروفایل کاملاً قابل‌شخصی‌سازی **Light (Day)** و **Dark (Night)** جابه‌جا می‌کند.

<div dir="ltr">

```
Light Mode  ──click tray icon──▶  Dark Mode
Dark Mode   ──click tray icon──▶  Light Mode
```

</div>

## ✨ ویژگی‌ها

- **یک کلیک = تغییر تم** — بدون باز کردن Settings ویندوز
- **آیکون هوشمند**: ماه وقتی ویندوز روشن است (کلیک → تیره)، خورشید وقتی تیره است (کلیک → روشن)
- **پروفایل‌های قابل‌شخصی‌سازی**: Windows Mode · App Mode · Transparency · Accent Color · Wallpaper
- **Theme Editor با پیش‌نمایش زنده** قبل از ذخیره
- **منوی راست‌کلیک حرفه‌ای**: Switch · Current Theme · Light/Dark · Settings · Start with Windows · About · Exit
- **تشخیص خودکار تغییر خارجی**: اگر تم را از Settings ویندوز عوض کنید، برنامه فوری می‌فهمد (WM_SETTINGCHANGE)
- **Single Instance**: اجرای دوباره، پنجره نسخه فعال را نشان می‌دهد
- **Start with Windows** بدون Administrator
- **Onboarding ۳ مرحله‌ای** در اولین اجرا
- **ذخیره‌سازی دائمی** در `%APPDATA%\com.themeswitcher.app\config.json` (نوشتن atomic)
- **Error Handling**: پیام خطا + Retry / Open Windows Settings / Copy diagnostics
- **زبان فارسی/انگلیسی** با پشتیبانی کامل RTL
- **معماری Auto-Update** (tauri-plugin-updater با امضای minisign)
- **بدون Administrator** — تمام عملیات در HKCU (per-user)

## 🏗️ معماری

**Tauri 2 + React + TypeScript + Rust**

| لایه | تکنولوژی |
|---|---|
| UI | React 18 + TypeScript + CSS (Fluent/Win11-style) |
| Shell | Tauri 2 (tray-icon, single instance native, vibrancy) |
| Native | Rust + `windows` crate (Registry, WM_SETTINGCHANGE, SystemParametersInfo) |
| Config | JSON در AppData (atomic write) |
| Installer | NSIS (per-user, بدون admin) |

جزئیات کامل معماری، تحلیل APIهای ویندوز و جدول قابلیت‌ها: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)

## 📋 پیش‌نیازها (برای Build)

- **Node.js** ≥ 18 (تست‌شده با 22)
- **Rust** ≥ 1.77 (toolchain `x86_64-pc-windows-msvc`)
- **Visual Studio Build Tools** با C++ workload (برای MSVC)
- **WebView2 Runtime** (از قبل روی Windows 11 و Windows 10 به‌رو نصب است)

## 🚀 دستورهای سریع

```bash
# نصب وابستگی‌ها
npm install

# اجرای حالت توسعه (debug)
npm run tauri:dev

# تست واحد Rust
npm run test:rust

# تایپ‌چک فرانت‌اند + بیلد
npm run build

# ساخت نصب‌کننده تولیدی (NSIS)
npm run tauri:build
```

خروجی build: `src-tauri/target/release/bundle/nsis/Windows Theme Switcher_1.0.0_x64-setup.exe`

## 📦 نصب

۱. فایل نصب‌کننده NSIS را اجرا کنید (per-user، بدون نیاز به Administrator).
۲. برنامه مستقیماً در System Tray کنار ساعت باز می‌شود.
۳. اولین اجرا: Onboarding ۳ مرحله‌ای → سپس برنامه به Tray منتقل می‌شود.

## 🖱️ استفاده

| تعامل | نتیجه |
|---|---|
| کلیک چپ روی آیکون tray | جابه‌جایی فوری Light ↔ Dark |
| راست‌کلیک | منوی کامل (Switch / Profiles / Settings / Autostart / About / Exit) |
| Settings → Light (Day) | ویرایش پروفایل روشن + پیش‌نمایش زنده |
| Settings → Dark (Night) | ویرایش پروفایل تیره + پیش‌نمایش زنده |
| Settings → Advanced | کلیدهای registry · Diagnostics · Restore defaults · Updates |

## 🔒 امنیت

- بدون elevated privileges — همه‌چیز در `HKCU`
- بدون telemetry، بدون ذخیره اطلاعات حساس
- CSP سخت‌گیرانه (`default-src 'self'`)
- اعتبارسنجی کامل config سمت Rust (serde + validation)
- کلید خصوصی امضا هرگز در repo نیست (`.gitignore`)

## 🗺️ نقشه راه (معماری آماده)

Auto Day/Night · Schedule · Sunrise/Sunset · Hotkeys · Custom profiles (Work/Gaming/Focus/Reading) · Wallpaper switching · Brightness · Night Light integration

## 📄 لایسنس

این پروژه تحت **GNU General Public License v3.0 (GPL-3.0)** منتشر می‌شود — یک لایسنس **کپی‌لفت** قوی.

- استفاده، مطالعه، تغییر و توزیع آزاد است.
- هر نسخهٔ تغییریافته یا مشتق‌شده باید **تحت همان لایسنس GPL-3.0** و **همراه با کد منبع** توزیع شود.
- هیچ‌کس نمی‌تواند این پروژه (یا مشتقات آن) را به نرم‌افزار اختصاصی/بسته تبدیل کند.

متن کامل: [`LICENSE`](LICENSE) — Copyright © 2026 DashAkoul

<div dir="ltr">

```
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.
```

</div>
