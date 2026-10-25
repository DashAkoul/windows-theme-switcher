# Testing Guide — Windows Theme Switcher

این سند روش تست هر بخش از برنامه را توضیح می‌دهد.

## 1. تست‌های خودکار (Rust unit tests)

```bash
npm run test:rust
```

تست‌های موجود:

| ماژول | تست |
|---|---|
| `config.rs` | roundtrip سریالایز، اعتبارسنجی hex color، ذخیره/بازیابی فایل، بازیابی از JSON خراب، پروفایل‌های پیش‌فرض |
| `theme.rs` | تبدیل `hex ↔ ABGR` (مثلاً `#0078D4 ↔ 0xFFD47800`)، خواندن وضعیت فعلی، ساخت پروفایل از وضعیت فعلی |
| `registry.rs` | خواندن build number ویندوز، مقادیر missing → None، roundtrip نوشتن/خواندن (کلید scratch) |
| `autostart.rs` | قالب‌بندی مقدار Run (quote شده)، roundtrip فعال/غیرفعال (با بازگردانی state قبلی) |
| `monitor.rs` | خواندن رشته wide، پایداری id پیام wake |
| `tray.rs` | اعتبارbeing آیکون‌های tray (PNG decode)، محتوای tooltip، ترجمه منو |
| `logger.rs` | الگوریتم تاریخ میلادی، قالب timestamp، tail لاگ |

## 2. تایپ‌چک فرانت‌اند

```bash
npm run build   # tsc --noEmit && vite build
```

## 3. بیلد تولیدی

```bash
npm run tauri:build
```

خروجی:
- `src-tauri/target/release/bundle/nsis/Windows Theme Switcher_1.0.0_x64-setup.exe`
- `src-tauri/target/release/Windows Theme Switcher.exe` (نسخه portable)

## 4. تست دستی (Smoke Test)

### ۴.۱ اولین اجرا (Onboarding)
1. فایل نصبی را اجرا کنید (یا exe را مستقیاجرا کنید).
2. ✅ پنجره Onboarding با ۳ اسلاید نمایش داده می‌شود.
3. ✅ دکمه‌های Next/Back/Skip کار می‌کنند.
4. روی «Done» یا «Open Settings» کلیک کنید.
5. ✅ برنامه در System Tray کنار ساعت visible است (آیکون ماه یا خورشید).

### ۴.۲ toggle سریع
1. وضعیت فعلی ویندوز را note کنید (Settings → Personalization → Colors).
2. روی آیکون tray **کلیک چپ** کنید.
3. ✅ تم ویندوز فوری عوض می‌شود (بدون باز شدن Settings).
4. ✅ آیکون tray عوض می‌شود (ماه ↔ خورشید).
5. ✅ راست‌کلیک → منو، وضعیت فعلی (● Dark Mode / ● Light Mode) درست است.

### ۴.۳ تشخیص تغییر خارجی
1. تم را از **Settings ویندوز** عوض کنید (Personalization → Colors → Dark).
2. ✅ آیکون tray و tooltip برنامه ظرف چند ثانیه به‌رو می‌شوند (بدون کلیک).
3. ✅ اگر پنجره Settings برنامه باز باشد، وضعیت current theme به‌رو می‌شود.

### ۴.۴ Single Instance
1. برنامه را اجرا کنید (در tray).
2. دوباره exe را اجرا کنید.
3. ✅ پروسه دوم ساخته نمی‌شود؛ پنجره Settings نسخه اول نمایش داده می‌شود.

### ۴.۵ Start with Windows
1. Settings → General → «Start with Windows» را فعال کنید.
2. از ویندوز sign out و دوباره sign in کنید (یا Task Manager → Startup).
3. ✅ برنامه مستقیماً در tray اجرا می‌شود، بدون پنجره اضافی.
4. ✅ تیک منوی tray («Start with Windows») فعال است.

### ۴.۶ Theme Editor و Preview
1. Settings → Light (Day) → تغییرات بدهید (مثلاً App Mode = Dark).
2. ✅ Preview زنده آیکون‌ها و رنگ‌ها را نشان می‌دهد.
3. «Save» → ✅ ذخیره می‌شود (badge «Saved»).
4. «Apply now» → ✅ ویندوز فوری تغییر می‌کند.
5. «Detect current» → ✅ تنظیمات فعلی ویندوز در ادیتور load می‌شود.

### ۴.۷ Wallpaper
1. در یک پروفایل، wallpaper انتخاب کنید (Browse…).
2. ✅ Preview تصویر را به‌عنوان دسکتاپ نشان می‌دهد.
3. Apply → ✅ پس‌زمینه دسکتاپ عوض می‌شود.

### ۴.۸ خطا
1. (اختیاری) دسترسی به Personalize را محدود کنید (سخت).
2. ✅ Toast خطا با دکمه‌های Retry / Open Windows Settings / Copy diagnostics نمایش داده می‌شود.
3. ✅ برنامه crash نمی‌کند.

### ۴.۹ فارسی/RTL
1. Settings → General → Language = فارسی.
2. ✅ کل UI راست‌به‌چپ می‌شود، منوی tray هم فارسی است.

## 5. رفع اشکال رایج

| علامت | علت | راه‌حل |
|---|---|---|
| آیکون tray دیده نمی‌شود | آیکون‌های hidden در taskbar settings | Settings → Personalization → Taskbar → Other system tray icons → enable |
| WebView2 missing | ویندوز خیلی قدیمی | نصب‌کننده bootstrapper آن را نصب می‌کند؛ یا از mستقیم microsoft.com |
| build fails: MSVC | Build Tools نصب نیست | نصب VS Build Tools با C++ workload |
| accent color فوری اعمال نشد | محدودیت مستند ویندوز | خروج/ورود مجدد؛ بقیه تنظیمات فوری هستند |
| `cargo test` روی sandbox fail | نوشتن در registry | تست‌ها scratch key می‌سازند و پاک می‌کنند؛ در CI مجاز باشد |

## 6. لاگ‌ها

- فایل لاگ: `%APPDATA%\com.themeswitcher.app\app.log`
- config: `%APPDATA%\com.themeswitcher.app\config.json`
- diagnostics کامل: Settings → Advanced → Copy diagnostics
