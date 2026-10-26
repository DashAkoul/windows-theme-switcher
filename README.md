# Windows Theme Switcher

**سوییچر تم ویندوز** — یک ابزار سبک System Tray برای Windows 10/11 که با **یک کلیک** روی آیکون کنار ساعت، تم ویندوز را بین Light و Dark جابه‌جا می‌کند.

<div dir="ltr">

```
Light  ──click tray icon──▶  Dark
Dark   ──click tray icon──▶  Light
```

</div>

## ✨ ویژگی‌ها (v2 — مینیمال)

- **یک کلیک = تغییر تم** — بدون باز کردن Settings ویندوز
- **آیکون هوشمند**: ماه وقتی ویندوز روشن است (کلیک → تیره)، خورشید وقتی تیره است (کلیک → روشن)
- **پاپ‌آپ کوچک (۳۴۰×۲۳۲)** — به‌جای پنجره کامل: فقط سوییچ Light/Dark، گزینه «Start with Windows» و لینک Settings ویندوز
- **مصرف رم در حد کیلوبایت**: در حالت tray فقط **~۱۵۰ کیلوبایت** working set (همان «Memory» در Task Manager) و ~۲.۳MB حافظه تخصیص‌یافته، با **صفر** پروسه `msedgewebview2.exe`. پنل با کلیک بیرون مخفی می‌شود و پس از ۲ دقیقه نابود می‌شود؛ دکمه ✕ فوراً آن را نابود می‌کند
- **تشخیص تغییر خارجی**: تغییر تم از Settings ویندوز فوراً شناسایی می‌شود (WM_SETTINGCHANGE)
- **Single Instance**: اجرای دوباره، پنل نسخه فعال را نشان می‌دهد
- **بدون Administrator** — همه عملیات در HKCU
- **Auto-Update** امضاشده (minisign) + نمایش لینک آپدیت در فوتر پنل
- **ذخیره‌سازی** در `%APPDATA%\com.themeswitcher.app\config.json` (نوشتن atomic)

## 📊 مصرف منابع (اندازه‌گیری‌شده)

| حالت | Working Set (Task Manager) | حافظه تخصیص‌یافته | CPU |
|---|---|---|---|
| فقط tray (پنل هرگز باز نشده) | **~۱۵۰KB** | ~۲.۳MB | ۰٪ |
| tray، بعد از یک‌بار باز کردن پنل | median **۱۴۸KB** (حداکثر جهش ~۱.۹MB) | ~۶.۹MB | ۰٪ |
| پنل باز | ~۴۹MB (پروسه اپ) + پروسه‌های WebView2 | — | فقط هنگام تعامل |

نکته مهم: این دو عدد را نباید قاطی کرد. **Working Set** صفحه‌هایی است که همین حالا در RAM مقیم‌اند (همان چیزی که Task Manager نشان می‌دهد) و با `EmptyWorkingSet` به حداقل می‌رسد؛ **حافظه تخصیص‌یافته (commit)** چیزی است که واقعاً به پروسه داده شده و لازم نیست مقیم باشد. هیچ داده‌ای حذف نمی‌شود — صفحه‌ها با اولین لمس برمی‌گردند.

روش کار در [`docs/ARCHITECTURE.md` §8.6](docs/ARCHITECTURE.md) توضیح داده شده است.

## 🏗️ معماری

**Tauri 2 + React + TypeScript + Rust**

| لایه | تکنولوژی |
|---|---|
| UI | React 18 + TypeScript (یک کامپوننت پنل، بدون routing و بدون i18n) |
| Shell | Tauri 2 (tray-icon، single instance native، پنل lazy) |
| Native | Rust + `windows` crate (Registry، WM_SETTINGCHANGE، SystemParametersInfo) |
| Config | JSON در AppData (atomic write) |
| Installer | NSIS (per-user، بدون admin) |

جزئیات: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · تست: [`docs/TESTING.md`](docs/TESTING.md)

## 📋 پیش‌نیازها (برای Build)

- **Node.js** ≥ 18 (تست‌شده با 22)
- **Rust** ≥ 1.77 (toolchain `x86_64-pc-windows-msvc`)
- **Visual Studio Build Tools** با C++ workload
- **WebView2 Runtime** (روی ویندوز ۱۱ و ویندوز ۱۰ به‌رو موجود است)

## 🚀 دستورهای سریع

```bash
npm install            # نصب وابستگی‌ها
npm run tauri:dev      # اجرای توسعه
npm run test:rust      # تست واحد Rust
npm run build          # تایپ‌چک + بیلد فرانت‌اند
npm run tauri:build    # ساخت نصب‌کننده NSIS
```

خروجی: `src-tauri/target/release/bundle/nsis/Windows Theme Switcher_2.1.0_x64-setup.exe`

## 🖱️ استفاده

| تعامل | نتیجه |
|---|---|
| کلیک چپ روی آیکون tray | جابه‌جایی فوری Light ↔ Dark |
| راست‌کلیک | Switch · Light / Dark · Open panel… · Start with Windows · Exit |
| «Open panel…» | پاپ‌آپ کوچک با سوییچ Light/Dark |
| بستن پاپ‌آپ | با کلیک بیرون یا ✕ — حافظه آزاد می‌شود |

## ⚙️ تنظیمات پیشرفته (اختیاری)

پروفایل‌های Light/Dark (نام، حالت ویندوز/اپ، شفافیت، رنگ تأکیدی، والپیپر) در `config.json` قابل ویرایش دستی هستند؛ رابط کاربری عمداً مینیمال نگه داشته شده است.

## 🔒 امنیت

- بدون elevated privileges — همه‌چیز در `HKCU`
- بدون telemetry
- CSP تنظیم‌شده در `tauri.conf.json` (با nonce خودکار Tauri)
- کلید خصوصی امضا هرگز در repo نیست

## 📄 لایسنس

**GNU GPL v3.0 (کپی‌لفت)** — استفاده، مطالعه، تغییر و توزیع آزاد است، مشروط بر اینکه نسخه‌های تغییریافته نیز تحت همان لایسنس و همراه کد منبع توزیع شوند.

متن کامل: [`LICENSE`](LICENSE) — Copyright © 2026 DashAkoul
