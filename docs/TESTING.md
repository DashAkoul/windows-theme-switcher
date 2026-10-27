# Testing Guide — Windows Theme Switcher v2

این سند روش تست هر بخش از برنامه را توضیح می‌دهد.

## 1. تست‌های خودکار (Rust unit tests)

```bash
npm run test:rust
```

تست‌های موجود:

| ماژول | تست |
|---|---|
| `config.rs` | roundtrip سریالایز، اعتبارسنجی hex color، ذخیره/بازیابی فایل، بازیابی از JSON خراب، پروفایل‌های پیش‌فرض |
| `theme.rs` | تبدیل `hex ↔ ABGR` (مثلاً `#0078D4 ↔ 0xFFD47800`)، خواندن وضعیت فعلی، ساخت پروفایل از وضعیت فعلی، اعمال idempotent |
| `registry.rs` | خواندن build number ویندوز، مقادیر missing → None، roundtrip نوشتن/خواندن (کلید scratch) |
| `autostart.rs` | قالب‌بندی مقدار Run (quote شده)، roundtrip فعال/غیرفعال (با بازگردانی state قبلی) |
| `monitor.rs` | خواندن رشته wide، پایداری id پیام wake |
| `tray.rs` | اعتبارسنجی آیکون‌های tray (PNG decode)، محتوای tooltip، ساخت منو |
| `logger.rs` | الگوریتم تاریخ میلادی، قالب timestamp |

## 2. تایپ‌چک فرانت‌اند

```bash
npm run build   # tsc --noEmit && vite build
```

## 3. بیلد تولیدی

```bash
npm run tauri build
```

خروجی:
- `src-tauri/target/release/bundle/nsis/Windows Theme Switcher_2.1.2_x64-setup.exe`
- `src-tauri/target/release/windows-theme-switcher.exe` (نسخه portable)

## 4. تست دستی (Smoke Test)

### ۴.۱ شروع در tray
1. فایل نصبی را اجرا کنید (یا exe را مستقیم اجرا کنید).
2. ✅ **هیچ پنجره‌ای** باز نمی‌شود؛ فقط آیکون tray ظاهر می‌شود.
3. ✅ مصرف حافظه در Task Manager حدود **۱۳MB** است (هیچ پروسه `msedgewebview2.exe` وجود ندارد).

### ۴.۲ باز کردن پاپ‌آپ
1. روی آیکون tray **کلیک چپ** کنید.
2. ✅ پنل **۳۴۰×۲۳۲** در گوشه پایین-راست، بالای تسک‌بار باز می‌شود.
3. ✅ محتوا: کارت‌های Light/Dark، «Start with Windows»، لینک «Windows theme settings…»، نسخه در فوتر.
4. ✅ هیچ متنی روی متن دیگر نمی‌افتد و چیزی بریده نمی‌شود.
5. روی جای دیگری از صفحه کلیک کنید → ✅ پنل مخفی می‌شود.

### ۴.۳ تعویض تم
1. وضعیت فعلی ویندوز را note کنید (Settings → Personalization → Colors).
2. در پنل روی کارت **Light** یا **Dark** کلیک کنید.
3. ✅ تم ویندوز **فوری** عوض می‌شود (زیر ۱ ثانیه).
4. ✅ کارت انتخاب‌شده highlight می‌شود و آیکون tray عوض می‌شود (ماه ↔ خورشید).
5. ✅ پنجره Not Responding نمی‌شود (قبلاً broadcast تا ۲۵ ثانیه بلاک می‌کرد).

### ۴.۴ تشخیص تغییر خارجی
1. تم را از **Settings ویندوز** عوض کنید (Personalization → Colors → Dark).
2. ✅ آیکون tray و tooltip ظرف چند ثانیه به‌رو می‌شوند (بدون کلیک).
3. ✅ اگر پنل باز باشد، کارت فعال به‌رو می‌شود.

### ۴.۵ Single Instance
1. برنامه در tray در حال اجراست.
2. دوباره exe را اجرا کنید.
3. ✅ پروسه دوم در حدود ۱-۲ ثانیه با کد ۰ خارج می‌شود و پنل نسخه اول باز می‌شود.

### ۴.۶ Start with Windows
1. در پنل، «Start with Windows» را فعال کنید.
2. از ویندوز sign out و دوباره sign in کنید (یا Task Manager → Startup).
3. ✅ برنامه مستقیماً در tray اجرا می‌شود، بدون پنجره.

### ۴.۷ آزادسازی حافظه (مهم)
1. برنامه را تازه اجرا کنید → در Task Manager حدود **۱۳MB** و **صفر** پروسه `msedgewebview2.exe`.
2. پنل را باز کنید → ۶ پروسه WebView ظاهر می‌شود (مصرف بالا).
3. روی **✕** کلیک کنید → ✅ پنجره فوراً نابود می‌شود و WebViewها همان لحظه صفر می‌شوند.
4. یا بیرون پنل کلیک کنید → ✅ پنل مخفی می‌شود و پس از **۲ دقیقه** نابود می‌شود
   (در لاگ: `panel closed after idle (memory released)`).
5. بعد از اولین باز شدن، مصرف اپ روی ~**۴۵MB** ثابت می‌ماند (runtime WebView2 در پروسه map می‌ماند) —
   این یک‌بار است و نشتی نیست.
6. چرخه باز/بسته را ۴ بار تکرار کنید: مصرف باید **ثابت** بماند و تعداد handle رشد محسوس نداشته باشد
   (`python scripts/cycle_test.py <exe> 4`).

### ۴.۸ آپدیتر
1. نسخه قدیمی‌تر را نصب کنید و پنل را باز کنید.
2. ✅ اگر نسخه جدیدتر منتشر شده باشد، در فوتر لینک «Update to vX» ظاهر می‌شود.
3. کلیک کنید → ✅ دانلود و نصب انجام می‌شود (امضا با کلید minisign بررسی می‌شود).

### ۴.۹ آیکون tray در حالت لایت
1. ویندوز را در **Light mode** بگذارید.
2. ✅ در ناحیه notification باید هلال **تیره** دیده شود (نه اسلات خالی).
3. ویندوز را Dark کنید → ✅ خورشید کهربایی دیده می‌شود.

### ۴.۱۰ خطا
1. (اختیاری) دسترسی به Personalize را محدود کنید (سخت).
2. ✅ پیام خطا به‌صورت یک خط متن قرمز داخل پنل نمایش داده می‌شود.
3. ✅ برنامه crash نمی‌کند و پنل بسته نمی‌شود.

### ۴.۱۱ سازگاری config قدیمی
1. یک `config.json` نسخه ۱ (با `language`, `uiTheme`, `animations`, `transparency`, `onboardingCompleted`) را در `%APPDATA%\com.themeswitcher.app\` بگذارید.
2. برنامه را اجرا کنید.
3. ✅ برنامه بدون خطا بالا می‌آید؛ فیلدهای حذف‌شده نادیده گرفته می‌شوند.
4. ✅ یک JSON خراب بگذارید → برنامه با config پیش‌فرض بالا می‌آید و فایل را بازسازی می‌کند.

## 5. رفع اشکال رایج

| علامت | علت | راه‌حل |
|---|---|---|
| آیکون tray دیده نمی‌شود | آیکون‌های hidden در taskbar settings | Settings → Personalization → Taskbar → Other system tray icons → enable |
| پنل باز نمی‌شود | پنجره focus را از دست داده | دوباره روی آیکون tray کلیک کنید |
| WebView2 missing | ویندوز خیلی قدیمی | نصب‌کننده bootstrapper آن را نصب می‌کند؛ یا مستقیم از microsoft.com |
| build fails: MSVC | Build Tools نصب نیست | نصب VS Build Tools با C++ workload |
| accent color فوری اعمال نشد | محدودیت مستند ویندوز | خروج/ورود مجدد؛ بقیه تنظیمات فوری هستند |
| `cargo test` روی sandbox fail | نوشتن در registry | تست‌ها scratch key می‌سازند و پاک می‌کنند؛ در CI مجاز باشد |

## 6. لاگ‌ها

- فایل لاگ: `%APPDATA%\com.themeswitcher.app\app.log`
- config: `%APPDATA%\com.themeswitcher.app\config.json`

نمونه لاگ سالم:

```
=== Windows Theme Switcher starting ===
starting in tray (no webview)
panel created
applied profile 'dark' in 412ms
panel hidden (focus lost)
panel destroyed (webview memory released)
```
