# Windows Theme Switcher — Architecture & Design Document

> نسخه: 1.0.0 · تاریخ: 2026-10-25 · وضعیت: تأییدشده

---

## 1. خلاصه اجرایی

**Windows Theme Switcher** یک تray-utility بسیار سبک برای Windows 10/11 است که با **یک کلیک** روی آیکون System Tray، حالت ظاهری ویندوز را بین دو پروفایل کاملاً قابل‌شخصی‌سازی **Light** و **Dark** جابه‌جا می‌کند.

اهداف طراحی:

| هدف | تعهد |
|---|---|
| Zero friction | تغییر تم = ۱ کلیک روی آیکون tray |
| Idle CPU ≈ 0 | بدون polling سنگین؛ رویدادمحور (WM_SETTINGCHANGE) |
| RAM پایین | < ~۱۲۰MB (باینری Rust + WebView2 مشترک سیستم) |
| Startup سریع | < ~۱ ثانیه تا readiness |
| بدون Administrator | تمام عملیات در HKCU (per-user) |
| بدون پنجره Settings ویندوز | فقط Registry + API رسمی + broadcast |

---

## 2. تصمیم معماری

### 2.1 مقایسه گزینه‌ها

| معیار | **Tauri 2 + React + Rust** ✅ | Electron | WinUI 3 / C# | WPF / C# | Native Win32 |
|---|---|---|---|---|---|
| RAM idle | ~۹۰–۱۲۰MB | ۱۵۰–۳۰۰MB | ~۶۰–۹۰MB | ~۵۰–۷۰MB | ~۱۵–۳۰MB |
| حجم باینری | ~۵–۱۰MB (+ WebView2 مشترک) | ~۱۵۰MB+ | ~۲۰–۶۰MB + Windows App SDK | ~۵–۱۵MB + .NET Runtime | ~۱MB |
| استارتاپ | سریع (~۳۰۰ms) | کند (۱–۳s) | متوسط | سریع | آنی |
| UI مدرن | ✅ React/CSS آزاد | ✅ | ✅ Fluent native | ⚠️ با styling سخت | ❌ دستی |
| Native OS integration | ✅ Rust + `windows` crate | ⚠️ N-API/FFI | ✅ | ✅ | ✅ |
| Auto-update رسمی | ✅ tauri-plugin-updater | ✅ electron-updater | ❌ دستی | ❌ دستی | ❌ دستی |
| Code signing / installer | ✅ NSIS/MSI | ✅ | ✅ MSIX | ✅ | ✅ |
| سرعت توسعه UI | ✅✅ | ✅ | ⚠️ | ⚠️ | ❌❌ |
| Maintainability | ✅ | ✅ | ✅ | ✅ | ❌ |
| نیاز به runtime جانبی | WebView2 (از قبل روی Win11 و Win10 به‌رو نصب است) | ندارد | Windows App SDK Runtime | .NET Desktop Runtime | ندارد |

### 2.2 دلیل انتخاب نهایی: Tauri 2 + React + TypeScript + Rust

1. **Performance و ROM/RAM**: تارگت کاربر «تقریباً نامحسوس بودن روی سیستم متوسط» است. الکترون این هدف را نقض می‌کند. تauri با بهره‌گیری از WebView2 (که از قبل روی ویندوز ۱۱ و اکثر ۱۰‌های به‌رو موجود است) حافظه بسیار کمتری می‌گیرد.
2. **Native layer با Rust**: همه کارهای Registry، broadcast پیام‌ها، tray icon، و single-instance در Rust و با `windows` crate رسمی مایکروسافت انجام می‌شود — بدون شبیه‌سازی کلیک، بدون automation شکننده.
3. **UI مدرن**: React + TypeScript + CSS آزاد → ظاهر Windows 11 Fluent-like (rounded corners، Segoe UI Variable) بدون محدودیت کنترل‌های قدیمی Win32. از v2 بک‌دراپ سیستمی (Mica/Acrylic) استفاده نمی‌شود: پنل خودش سطوح Fluent دارد و این مسیر روی پنجره غیرشفاف WebView2 می‌توانست ناحیه کلاینت را مشکی کند.
4. **Auto-update آماده**: `tauri-plugin-updater` با امضای minisign و endpoint روی GitHub Releases — معماری از روز اول آپدیت‌پذیر است.
5. **امنیت**: CSP، allowlist مجوزها (capabilities)، عدم نیاز به Administrator (نصب per-user).

> **نکته WebView2**: روی Windows 11 و Windows 10 نسخه‌های به‌رو (1803+) از قبل نصب است. نصب‌کننده با `downloadBootstrapper` در صورت نبودن، آن را نصب می‌کند.

---

## 3. تحلیل قابلیت‌ها: چه چیزی واقعاً از طریق API رسمی ممکن است؟

### 3.1 جدول matrix کامل

| # | قابلیت | روش فنی | نیاز به Admin | Windows 10 | Windows 11 | فوری؟ | محدودیت |
|---|---|---|---|---|---|---|---|
| 1 | **App Mode** (تم اپلیکیشن‌ها) | `HKCU\...\Themes\Personalize\AppsUseLightTheme` (DWORD) + broadcast | ❌ خیر | 1809+ | ✅ | ✅ | — |
| 2 | **Windows Mode** (تسک‌بار، منوی Start، ویجت‌ها) | `HKCU\...\Themes\Personalize\SystemUsesLightTheme` (DWORD) + broadcast | ❌ خیر | 1903+ | ✅ | ✅ | در بیلدهای قدیمی‌تر Win10 این کلید ممکن است وجود نداشته باشد → fallback: مقدار App Mode |
| 3 | **Transparency** (شفافیت پنجره‌ها/تسک‌بار) | `HKCU\...\Themes\Personalize\EnableTransparency` (DWORD) + broadcast | ❌ خیر | 1809+ | ✅ | ✅ | — |
| 4 | **Accent Color** | `HKCU\Software\Microsoft\Windows\DWM\AccentColor` (DWORD, قالب ABGR) + broadcast | ❌ خیر | ✅ | ✅ | ⚠️ نیمه‌فوری | برخی سطوح UI ممکن است نیاز به خروج/ورود مجدد داشته باشند؛ مستندات مایکروسافت API عمومی برای set ندارد (registry روش رسمیِ پذیرفته‌شده است) |
| 5 | **نمایش Accent روی تسک‌بار/Start** | `DWM\ColorPrevalence` (DWORD) | ❌ خیر | ✅ | ✅ | ⚠️ | مرتبط با #4 |
| 6 | **Wallpaper** | `SystemParametersInfoW(SPI_SETDESKWALLPAPER, …, SPIF_UPDATEINIFILE｜SPIF_SENDCHANGE)` | ❌ خیر | ✅ | ✅ | ✅ | فقط فایل محلی (jpg/png/bmp)؛ مسیر مطلق |
| 7 | **خواندن وضعیت فعلی تم** | خواندن همان کلیدهای Personalize + `Control Panel\Desktop\Wallpaper` | ❌ خیر | ✅ | ✅ | — | — |
| 8 | **تشخیص تغییر خارجی** (کاربر از Settings تغییر دهد) | پنجره native مخفی + گوش دادن به `WM_SETTINGCHANGE` ("ImmersiveColorSet") و `WM_DWMCOLORIZATIONCOLORCHANGED` | ❌ خیر | ✅ | ✅ | ✅ | رویدادمحور، بدون polling |
| 9 | **Start with Windows** | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (REG_SZ = مسیر exe) | ❌ خیر | ✅ | ✅ | — | per-user؛ بدون نیاز به Task Scheduler |
| 10 | **Night Light** | — | — | ⚠️ | ⚠️ | — | API عمومی رسمی برای toggle ندارد؛ فعلاً خارج از scope پایه |
| 11 | **Brightness / HDR** | WMI / DDC/CI | ⚠️ بستگی به درایور | ⚠️ | ⚠️ | — | نیاز به تست سخت‌افزاری؛ فاز آینده |
| 12 | **Monitor-specific profile** | — | — | ❌ | ❌ | — | ویندوز تم per-monitor ندارد؛ آینده فقط برای wallpaper |
| 13 | **Hotkey / Mouse shortcut** | `RegisterHotKey` / low-level hook | ❌ خیر | ✅ | ✅ | — | فاز آینده (معماری اجازه می‌دهد) |
| 14 | **Schedule / Sunset-Sunrise** | Timer + geo API | ❌ خیر | ✅ | ✅ | — | فاز آینده |

### 3.2 نکات کلیدی فنی

**Broadcast بعد از نوشتن Registry** — برای اعمال فوری:
```
SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0,
    L"ImmersiveColorSet", SMTO_ABORTIFHUNG, 5000, &result)
```
این دقیقاً همان کاری است که اپ Settings ویندوز انجام می‌دهد.

**فرمت AccentColor**: مقدار ABGR است. برای `#0078D4` مقدار DWORD برابر `0xFFD47800` می‌شود.

** fallback SystemUsesLightTheme**: اگر کلید وجود نداشت (Win10 خیلی قدیمی)، برنامه آن را برابر `AppsUseLightTheme` تلقی می‌کند.

**آنچه انجام نمی‌دهیم (به‌خاطر شکنندگی یا نبود API رسمی)**:
- شبیه‌سازی کلیک در Settings ویندوز (فرآیند brittle) ❌
- نوشتن در `HKLM` یا درخواست Administrator ❌ (هیچ قابلیت پایه‌ای نیاز ندارد)
- toggle مستقیم Night Light از طریق کلیدهای CloudStore (غیرمستند و شکننده) ❌

---

## 4. تشخیص وضعیت فعلی و تغییر خارجی

منبع حقیقت همیشه **Registry ویندوز** است، نه state داخلی برنامه:

1. **رویدادمحور (اصلی)**: یک thread اختصاصی Rust یک پنجره native مخفی با کلاس `WTS_SignalWindow` می‌سازد و حلقه پیام اجرا می‌کند. با دریافت `WM_SETTINGCHANGE`/`WM_DWMCOLORIZATIONCOLORCHANGED`، وضعیت از Registry خوانده و در صورت تغییر:
   - آیکون و tooltip و منوی tray به‌رو می‌شود؛
   - رویداد `system-theme-changed` به فرانت‌اند emit می‌شود.
2. **immediate**: هنگام باز شدن منوی tray و کلیک، وضعیت از نو خوانده می‌شود.
3. **هیچ polling دوره‌ای سنگینی وجود ندارد** → CPU ≈ 0 در idle.

---

## 5. ساختار پروژه

```
windows-theme-switcher/
├── package.json / tsconfig*.json / vite.config.ts
├── index.html                     # ورودی Vite (CSP سخت‌گیرانه)
├── docs/
│   ├── ARCHITECTURE.md            # همین سند
│   └── TESTING.md                 # تست و رفع اشکال
├── scripts/
│   └── generate_icons.py          # تولید آیکون‌ها (PNG/ICO، بدون وابستگی خارجی)
├── src/                           # فرانت‌اند React + TypeScript (v2: یک پنل)
│   ├── main.tsx
│   ├── App.tsx                    # بارگذاری state + رندر پنل
│   ├── types.ts                   # تایپ‌های مشتق از struct‌های Rust
│   ├── api.ts                     # پوشش typed روی invoke + listen (+ mock مرورگر)
│   ├── styles/global.css          # توکن‌های پنل (light/dark با prefers-color-scheme)
│   ├── components/controls.tsx    # Toggle + آیکون‌های خورشید/ماه
│   └── views/Panel.tsx            # تنها صفحه: سوییچ Light/Dark + دو گزینه
└── src-tauri/                     # بک‌اند Rust
    ├── Cargo.toml / build.rs / tauri.conf.json / capabilities/default.json
    ├── icons/                     # آیکون اپ + tray (moon/sun)
    └── src/
        ├── main.rs                # ورودی باینری (windows_subsystem)
        ├── lib.rs                 # run()، setup، ثبت پلاگین‌ها
        ├── error.rs               # AppError با Serialize + کدهای خطا
        ├── logger.rs              # لاگر فایل ساده در AppData
        ├── config.rs              # AppConfig/Persist + atomic write + تست
        ├── registry.rs            # خواندن/نوشتن DWORD/SZ + helperها + تست
        ├── theme.rs               # تشخیص/اعمال تم، broadcast، رنگ ABGR + تست
        ├── monitor.rs             # پنجره native: WM_SETTINGCHANGE + single-instance
        ├── autostart.rs           # Run key + تست منطق مسیر
        ├── tray.rs                # TrayIcon داینامیک + منوی متنی
        └── commands.rs            # فرمان‌های Tauri برای فرانت‌اند
```

---

## 6. جریان داده

```
┌──────────────────────────────────────────────┐
│ System Tray (Rust)                           │
│  کلیک چپ ──► commands::toggle_theme()        │
│  راست‌کلیک ──► منو: Switch/Light/Dark/...     │
└──────────────┬───────────────────────────────┘
               ▼
        theme::apply_profile(id)
               │
   ┌───────────┼───────────────────────────┐
   ▼           ▼           ▼                ▼
Registry   Registry   DWM Accent   SystemParametersInfo
Personalize Personalize (wallpaper)  SPI_SETDESKWALLPAPER
   └───────────┴───────────────────────────┘
               ▼
   SendMessageTimeoutW(WM_SETTINGCHANGE, "ImmersiveColorSet")
               ▼
   به‌رورسانی tray icon/tooltip + emit رویداد به UI
               ▼
   ذخیره config.json (atomic write)

WM_SETTINGCHANGE خارجی ──► monitor.rs ──► خواندن Registry
                                        ──► به‌رورسانی tray + emit
```

---

## 7. مدل پیکربندی (JSON در AppData)

مسیر: `%APPDATA%\com.themeswitcher.app\config.json` — نوشتن atomic (نوشتن در فایل موقت + rename) برای جلوگیری از corruption.

```json
{
  "version": 1,
  "settings": {
    "startWithWindows": false,
    "checkUpdates": true
  },
  "profiles": {
    "light": {
      "name": "Day",
      "windowsMode": "light",
      "appMode": "light",
      "transparency": true,
      "accentColor": null,
      "wallpaper": null
    },
    "dark": {
      "name": "Night",
      "windowsMode": "dark",
      "appMode": "dark",
      "transparency": true,
      "accentColor": null,
      "wallpaper": null
    }
  },
  "lastActiveProfile": "dark"
}
```

- `profiles` از نوع `HashMap<String, ThemeProfile>` است تا افزودن پروفایل‌های Work/Gaming/Reading در آینده بدون migration باشد.
- `accentColor: null` و `wallpaper: null` به معنای «تغییر نده» هستند.
- در v2 رابط کاربری فقط Light/Dark را عوض می‌کند؛ این فیلدها برای ویرایش دستی/آینده باقی مانده‌اند
  و فیلدهای حذف‌شده‌ی نسخه ۱ (`language`, `uiTheme`, `animations`, `transparency`, `onboardingCompleted`)
  توسط serde نادیده گرفته می‌شوند (config قدیمی بدون migration کار می‌کند).
- اعتبارسنجی: مقادیر enum فقط light/dark؛ رنگ با regex hex؛ wallpaper باید مسیر مطلق موجود باشد (در زمان اعمال).

---

## 8. UX طراحی

### 8.1 System Tray

| تعامل | رفتار |
|---|---|
| کلیک چپ | toggle فوری: light ↔ dark (پروفایل مخالف اعمال می‌شود) |
| راست‌کلیک | منوی متنی |

منوی راست‌کلیک:
```
Switch to Dark (Night)     ← اکشن اصلی
● Dark Mode                ← وضعیت فعلی (disabled)
─────────────────────────
○ Light (Day)              ← radio
● Dark (Night)
─────────────────────────
Settings
Start with Windows         ← checkable
─────────────────────────
About
Exit
```

**آیکون داینامیک** (کاربر از آیکون می‌فهمد کلیک چه می‌کند):
- ویندوز Light است → آیکون **Moon** (تیره، خوانا روی تسک‌بار روشن) → کلیک = رفتن به Dark
- ویندوز Dark است → آیکون **Sun** (کهربایی، خوانا روی تسک‌بار تیره) → کلیک = رفتن به Light

### 8.2 پنل پاپ‌آپ (تنها رابط کاربری)

پنجرهٔ ثابت **۳۴۰×۲۳۲**، بدون decorations، `always_on_top`، `skip_taskbar`:

```
┌──────────────────────────────────┐
│ ◐ Windows Theme Switcher      ✕ │  ← drag region
├──────────────────────────────────┤
│  ┌──────────┐   ┌──────────┐     │
│  │  ☀ Light │   │  ☾ Dark  │     │  ← انتخاب فعال با border/tint اکسنت
│  └──────────┘   └──────────┘     │
│  Start with Windows         [○]  │
│  Windows theme settings…         │
│  v2.0.0                          │
└──────────────────────────────────┘
```

- با کلیک بیرون (از دست دادن focus) مخفی می‌شود؛ پس از ۲ دقیقه بیکاری **destroy** می‌شود تا حافظه WebView آزاد شود.
- پنجره در `tauri.conf.json` تعریف نشده؛ `ensure_panel()` آن را lazy می‌سازد و در گوشه پایین-راست work area قرار می‌دهد.
- اگر آپدیت موجود باشد، در فوتر یک لینک «Update to vX» ظاهر می‌شود.
- خطاها به‌صورت یک خط متن قرمز داخل پنل نمایش داده می‌شوند (بدون toast/dialog).

### 8.5 Error UX

اگر نوشتن Registry شکست بخورد، خطا به‌صورت **یک خط متن قرمز داخل خود پنل** زیر کارت‌ها نمایش داده می‌شود (بدون toast، dialog یا پنجره جدا). پیام خطا از `AppError` می‌آید و کد خطا + متن کوتاه را نشان می‌دهد؛ در صورت نیاز لینک «Windows theme settings…» کاربر را به مسیر درست هدایت می‌کند. برنامه crash نمی‌کند و پنل باز می‌ماند.

### 8.6 مدیریت حافظه (Working Set)

دو عدد متفاوت وجود دارد و باید جدا دیده شوند:

| سنجه | معنی | مقدار این برنامه |
|---|---|---|
| **Working Set** (همان «Memory» در Task Manager) | صفحه‌هایی که همین حالا در RAM فیزیکی مقیم هستند | **~۲۰۰–۳۰۰KB** در حالت بیکار |
| **Private Commit** | حافظه‌ای که واقعاً به پروسه تخصیص یافته (لازم نیست مقیم باشد) | ~۲.۴MB |

**مکانیزم:** پروسه در حالت tray تقریباً همیشه بیکار است، ولی هر صفحه‌ای که هنگام استارتاپ یا باز شدن پنل لمس شود، مقیم می‌ماند؛ ویندوز خودش آن‌ها را پس نمی‌گیرد. ماژول `memory.rs` با `EmptyWorkingSet` از مدیر حافظه می‌خواهد این صفحه‌ها را به standby list منتقل کند. هیچ داده‌ای از دست نمی‌رود: commit ثابت می‌ماند و صفحه‌ها با اولین لمس برمی‌گردند.

نکته: یک بار trim حدود ۱MB باقی می‌گذارد، چون خود فراخوانی و teardown نخ thread چند صفحه را لمس می‌کند؛ به همین دلیل `trim_settle()` دو بار (با ۳۵۰ms فاصله) trim می‌کند و به کف ~۲۰۰KB می‌رسد.

**زمان‌بندی trim (رویدادمحور، نه تایمر پرتکرار):**

| رویداد | دلیل |
|---|---|
| ۱.۵ ثانیه بعد از پایان setup | جهش استارتاپ (۱۳MB) آزاد شود |
| بعد از destroy شدن پنل | runtime WebView2 که map شده بود صفحه‌بندی شود (۴۵MB → ~۲۰۰KB) |
| بعد از hide شدن پنل (کلیک بیرون) | در آن ۱۲۰ ثانیه‌ای که منتظر destroy است، حافظه بالا نماند |
| بعد از هر آیتم منوی tray / کلیک چپ | منو rebuild و آیکون swap می‌شود |
| janitor هر ۳ ثانیه | هر صفحه‌ای که در این فاصله لمس شد، دوباره آزاد شود |

janitor روی thread موجود «انتظار برای wake event» سوار شده است: به‌جای `WaitForSingleObject(INFINITE)` با تایم‌اوت ۳ ثانیه منتظر می‌ماند و در حالت timeout یک trim می‌زند. بنابراین **هیچ thread جدیدی** اضافه نشده و هزینه CPU آن عملاً صفر است (اندازه‌گیری: ۰.۰۰ ثانیه CPU در ۳۰ ثانیه بیکاری).

هزینه این کار: اولین کلیک روی tray بعد از یک دوره بیکاری ممکن است چند میلی‌ثانیه page-fault داشته باشد — در مقابل، ابزار در Task Manager عملاً دیده نمی‌شود.

---

## 9. امنیت

| مورد | پیاده‌سازی |
|---|---|
| حداقل دسترسی | تمام عملیات در `HKCU`؛ بدون elevation؛ نصب NSIS با `installMode: currentUser` |
| بدون اطلاعات حساس | config فقط تنظیمات ظاهری؛ هیچ credential/telemetry |
| جلوگیری از DLL hijacking | بدون ثبت DLL سفارشی؛ فقط crate‌های رسمی؛ فرآیند باینری امضا‌شده در Release |
| اعتبارسنجی config | schema سمت Rust (serde) + اعتبارسنجی enum/hex/مسیر پیش از اعمال |
| دستورات Shell ناامن | فقط `ShellExecuteW` با URI ثابت `ms-settings:` — بدون ترکیب ورودی کاربر |
| Code signing | `.tauri/*.key` در gitignore؛ Release با امضای minisign؛ مستندات CI |
| Auto-update امن | `tauri-plugin-updater` با pubkey در conf؛ بررسی صحت فایل (امضا) پیش از نصب؛ rollback با نگهداری نسخه قبلی توسط NSIS |
| CSP | در `tauri.conf.json` (`app.security.csp`) تنظیم می‌شود، **نه** به‌صورت `<meta>` در HTML. دلیل: Tauri اسکریپت راه‌انداز IPC را inline تزریق می‌کند و یک meta CSP بدون nonce آن را بلاک می‌کرد (باگ صفحه مشکی v1.0.0). بدون remote script/font. |
| Single instance | Named Mutex + FindWindow + SendMessage پیام ثبت‌شده (بدون pipe/socket) |

---

## 10. آپدیت خودکار (معماری آماده)

- پلاگین `tauri-plugin-updater` در بک‌اند و `@tauri-apps/plugin-updater` در فرانت‌اند.
- endpoint: `https://github.com/DashAkoul/windows-theme-switcher/releases/latest/download/latest.json`.
- کلید عمومی minisign در `tauri.conf.json`؛ کلید خصوصی هرگز در repo نیست (`.gitignore`).
- بررسی آپدیت **بی‌صدا** است: هنگام باز شدن پنل یک `check()` در پس‌زمینه اجرا می‌شود و فقط اگر نسخه جدیدتری وجود داشته باشد، در فوتر لینک «Update to vX» ظاهر می‌شود. کلیک روی آن دانلود + تأیید امضا + نصب را انجام می‌دهد.
- با `settings.checkUpdates = false` این بررسی کاملاً غیرفعال می‌شود (هیچ درخواست شبکه‌ای زده نمی‌شود).
- CI (آینده): GitHub Action روی tag → build → sign → تولید `latest.json` + NSIS در Release.

---

## 11. فازهای پیاده‌سازی

| فاز | محتوا | برنچ |
|---|---|---|
| 0 | اسکلت + مستندات | `main` |
| 1 | Rust core: config/registry/theme/monitor + تست واحد | `feature/core-tray` |
| 2 | Tray + commands + main wiring + single instance | `feature/core-tray` |
| 3 | فرانت‌اند v1: onboarding + settings + editor + preview + i18n | `feature/settings-ui` |
| 4 | آیکون‌ها + updater + تست نهایی + build | `feature/polish` |
| 5 | merge به `main` + tag `v1.0.0` + push | — |
| 6 | v1.0.1: رفع هنگ، مصرف رم، تغییر زبان، سیل خطاها | `fix/performance-i18n` |
| 7 | v1.0.2: رفع آیکون شفاف tray در Light mode | `fix/tray-icon-light` |
| 8 | v2.0.0: پاپ‌آپ مینیمال، حذف i18n و صفحات اضافی، lazy window | `feat/v2-minimal-panel` |

هر فاز با commit معنادار، تست (`cargo test` + `tsc --noEmit` + build) و merge بدون fast-forward به `develop` انجام می‌شود.

---

## 12. تست

- **تست واحد Rust** (`cargo test`): تبدیل رنگ ABGR، serialize/deserialize config، اعتبارسنجی پروفایل، منطق fallback `SystemUsesLightTheme`، مسیرهای registry.
- **type-check فرانت‌اند**: `tsc --noEmit && vite build`.
- **build تولید**: `npm run tauri build` → نصب‌کننده NSIS در `src-tauri/target/release/bundle/nsis/`.
- **smoke test**: اجرای باینری → بررسی زنده‌ماندن پروسه + نوشتن لاگ + بسته‌شدن تمیز.
- **تست دستی**: جزئیات در `docs/TESTING.md` (تشخیص تغییر خارجی، single instance، autostart، preview).
