# Testing Guide — Windows Theme Switcher v2

این سند روش تست هر بخش از برنامه را توضیح می‌دهد. همهٔ اعداد این سند از اجرای واقعی روی
ویندوز ۱۱ (build 26300، نمایشگر ۲۰۰٪) گرفته شده‌اند.

## 1. تست‌های خودکار (Rust unit tests)

```bash
npm run test:rust
```

| ماژول | تست‌ها |
|---|---|
| `config.rs` | roundtrip سریالایز، اعتبارسنجی `#RRGGBB`، ذخیره/بازیابی فایل، بازیابی از JSON خراب، **بازیابی از JSON سالم با مقدار نامعتبر**، نادیده‌گرفتن فیلدهای نسخه‌های قدیمی، پروفایل‌های پیش‌فرض |
| `theme.rs` | تبدیل `hex ↔ ABGR` (`#0078D4 ↔ 0xFFD47800`)، **رد کردن ورودی غیر-ASCII بدون پنیک**، خواندن وضعیت، ساخت پروفایل از وضعیت فعلی، اعمال idempotent |
| `registry.rs` | build number ویندوز، مقادیر missing → `None`، roundtrip نوشتن/خواندن روی کلید scratch |
| `autostart.rs` | قالب مقدار Run (quote شده)، roundtrip فعال/غیرفعال، **بازنویسی مسیر قدیمی توسط `sync`** |
| `monitor.rs` | خواندن رشتهٔ wide، **رد کردن اشاره‌گر غیرقابل‌خواندن**، پایداری id پیام wake |
| `tray.rs` | اعتبارسنجی آیکون‌ها (PNG decode)، محتوای tooltip، حالت پیش‌فرض |
| `logger.rs` | الگوریتم تاریخ میلادی، قالب timestamp، **چرخش لاگ بزرگ** |
| `memory.rs` | فراخوانی trim بدون پنیک |

تست‌هایی که Run key واقعی را می‌نویسند با یک `Mutex` سریالایز شده‌اند (Rust تست‌ها را موازی اجرا می‌کند).

## 2. تایپ‌چک و بیلد فرانت‌اند

```bash
npm run build   # tsc --noEmit && vite build
```

## 3. بیلد تولیدی

```bash
npm run tauri build
```

خروجی:
- `src-tauri/target/release/bundle/nsis/Windows Theme Switcher_<version>_x64-setup.exe`
- `src-tauri/target/release/windows-theme-switcher.exe` (نسخهٔ portable)

اسکریپت دود (اختیاری، بعد از بیلد):

```bash
bash scripts/smoke_test.sh
```

## 4. تست دستی

### ۴.۱ شروع در tray
1. exe را اجرا کنید.
2. ✅ **هیچ پنجره‌ای** باز نمی‌شود؛ فقط آیکون tray ظاهر می‌شود.
3. ✅ در Task Manager مصرف اپ **~۱۴۸KB** است و **هیچ** پروسهٔ `msedgewebview2.exe` مربوط به این
   برنامه وجود ندارد.

### ۴.۲ باز کردن پاپ‌آپ
1. روی آیکون tray **کلیک چپ** کنید.
2. ✅ پنل **۳۴۰×۲۳۲** در گوشهٔ پایین-راست، بالای تسک‌بار باز می‌شود (در ~۱ ثانیه).
3. ✅ پنل **باز می‌ماند** و بلافاصله مخفی نمی‌شود (در لاگ: `ignoring early focus loss (…ms after show)`).
4. ✅ محتوا: کارت‌های Light/Dark، «Start with Windows»، لینک «Windows theme settings…»، نسخه در فوتر.
5. ✅ هیچ متنی روی متن دیگر نمی‌افتد و چیزی بریده نمی‌شود.
6. کلیک بیرون پنل → ✅ مخفی می‌شود.

### ۴.۳ چند-مانیتوری
1. با دو نمایشگر، پنل را از مانیتوری باز کنید که تسک‌بار/آیکون tray روی آن است.
2. ✅ پنل روی **همان مانیتور** (مانیتور زیر نشانگر ماوس) باز می‌شود، نه همیشه روی مانیتور اصلی.

### ۴.۴ تعویض تم
1. وضعیت فعلی ویندوز را note کنید (Settings → Personalization → Colors).
2. در پنل روی کارت **Light** یا **Dark** کلیک کنید.
3. ✅ تم ویندوز فوری عوض می‌شود (لاگ: `applied profile 'dark' in 1ms`).
4. ✅ کارت انتخاب‌شده highlight می‌شود و آیکون tray عوض می‌شود (ماه ↔ خورشید).
5. ✅ پنجره Not Responding نمی‌شود (broadcast روی thread جداگانه و fire-and-forget است).

### ۴.۵ تشخیص تغییر خارجی
1. تم را از **Settings ویندوز** عوض کنید.
2. ✅ آیکون tray و tooltip ظرف چند ثانیه به‌رو می‌شوند (بدون کلیک).
3. ✅ اگر پنل باز باشد، کارت فعال به‌رو می‌شود.

### ۴.۶ Single Instance
1. برنامه در tray در حال اجراست.
2. دوباره exe را اجرا کنید.
3. ✅ پروسهٔ دوم در حدود ۱-۲ ثانیه با کد ۰ خارج می‌شود و پنل نسخهٔ اول باز می‌شود.

### ۴.۷ Start with Windows
1. در پنل یا منوی tray «Start with Windows» را فعال کنید.
2. ✅ تیک منوی tray همان لحظه به‌روز می‌شود.
3. Task Manager → Startup → ✅ ورودی فعال است و به مسیر فعلی exe اشاره می‌کند.

### ۴.۸ آزادسازی حافظه
| مرحله | Working Set | Private commit |
|---|---|---|
| شروع سرد (فقط tray) | ~۱۴۸KB | ~۲.۳MB |
| پنل باز | ~۴۹MB + پروسه‌های WebView2 | — |
| بعد از بستن پنل (median در ۳۰ ثانیه) | ~۱۴۸KB | ~۶.۹MB |

1. ✅ روی **✕** کلیک کنید → پنجره فوراً نابود می‌شود و WebViewها همان لحظه صفر می‌شوند.
2. ✅ یا بیرون کلیک کنید → پنل مخفی و پس از **۲ دقیقه** نابود می‌شود
   (لاگ: `panel closed after idle (memory released)`).
3. ✅ بعد از اولین باز شدن پنل، private commit روی ~۶.۹MB ثابت می‌ماند (باقی‌ماندهٔ runtime
   WebView2 در پروسه) — این یک‌بار است و نشتی نیست.
4. چرخهٔ باز/بسته را ۴ بار تکرار کنید (`python scripts/cycle_test.py <exe> 4`):
   ✅ RAM باید **ثابت** بماند و WebViewها هر چرخه صفر شوند.

> نکته: Working Set همان «Memory» در Task Manager است و با `EmptyWorkingSet` پایین نگه داشته
> می‌شود؛ private commit حافظهٔ تخصیص‌یافته است و لازم نیست مقیم باشد. توضیح کامل در
> `docs/ARCHITECTURE.md` §8.6.

### ۴.۹ آپدیتر
1. نسخهٔ قدیمی‌تر را نصب کنید و پنل را باز کنید.
2. ✅ اگر نسخهٔ جدیدتر منتشر شده باشد، در فوتر لینک «Update to vX» ظاهر می‌شود.
3. کلیک کنید → ✅ دانلود + بررسی امضای minisign + نصب (پروسه با `std::process::exit` بسته می‌شود).

### ۴.۱۰ آیکون tray در حالت لایت
1. ویندوز را در **Light mode** بگذارید.
2. ✅ در ناحیهٔ notification باید هلال **تیره** دیده شود (نه اسلات خالی).
3. ویندوز را Dark کنید → ✅ خورشید کهربایی دیده می‌شود.

### ۴.۱۱ خطا
1. (اختیاری) دسترسی به Personalize را محدود کنید (سخت).
2. ✅ پیام خطا به‌صورت متن قرمز داخل پنل (حداکثر دو خط) نمایش داده می‌شود.
3. ✅ برنامه crash نمی‌کند و پنل بسته نمی‌شود.

### ۴.۱۲ سازگاری config قدیمی
1. یک `config.json` نسخه ۱ (با `language`, `uiTheme`, `animations`, `onboardingCompleted`) یا
   نسخهٔ ۲.۰ (با `lastActiveProfile`) بگذارید.
2. ✅ برنامه بدون خطا بالا می‌آید؛ فیلدهای حذف‌شده نادیده گرفته می‌شوند.
3. ✅ یک JSON خراب بگذارید → با config پیش‌فرض بالا می‌آید و فایل را در `config.json.corrupt`
   پشتیبان می‌گیرد.
4. ✅ یک JSON سالم با مقدار نامعتبر بگذارید (مثلاً `"name": "   "`) → برنامه **بالا می‌آید**،
   فایل در `config.json.invalid` پشتیبان و config پیش‌فرض بازسازی می‌شود
   (لاگ: `config values invalid (…); using defaults`).

### ۴.۱۳ چرخش لاگ
1. `app.log` را تا بیش از ۲۵۶KB پر کنید (یا چند بار تم را عوض کنید).
2. ✅ فایل به `app.log.1` منتقل و لاگ تازه شروع می‌شود.

## 5. رفع اشکال رایج

| علامت | علت | راه‌حل |
|---|---|---|
| آیکون tray دیده نمی‌شود | آیکون‌های hidden در تنظیمات تسک‌بار | Settings → Personalization → Taskbar → Other system tray icons |
| پنل باز نمی‌شود | یک نسخهٔ قدیمی هنوز در tray است | از منوی tray → Exit، بعد دوباره اجرا کنید |
| WebView2 missing | ویندوز خیلی قدیمی | نصب‌کننده bootstrapper آن را نصب می‌کند |
| build fails: MSVC | Build Tools نصب نیست | نصب VS Build Tools با C++ workload |
| accent color فوری اعمال نشد | محدودیت مستند ویندوز | خروج/ورود مجدد؛ بقیهٔ تنظیمات فوری هستند |
| `cargo test` روی CI fail | نوشتن در registry/Run key | تست‌ها state را بازمی‌گردانند؛ در محیط sandbox ممکن است skip شوند |

## 6. لاگ‌ها

- لاگ: `%APPDATA%\com.themeswitcher.app\app.log` (چرخش در ۲۵۶KB به `app.log.1`)
- config: `%APPDATA%\com.themeswitcher.app\config.json`
- پشتیبان‌ها: `config.json.corrupt` (JSON خراب) و `config.json.invalid` (مقدار نامعتبر)

زمان‌ها **UTC** هستند.

نمونهٔ لاگ سالم (شروع در tray، باز کردن پنل، تعویض تم):

```
2026-10-27 16:32:13 [INFO] === Windows Theme Switcher starting ===
2026-10-27 16:32:13 [INFO] starting in tray (panel is created on demand)
2026-10-27 16:32:13 [INFO] startup complete (build 26300, config: …\config.json)
2026-10-27 16:32:41 [INFO] panel shown
2026-10-27 16:32:41 [INFO] ignoring early focus loss (16ms after show)
2026-10-27 16:32:44 [INFO] applied profile 'dark' in 1ms
2026-10-27 16:32:50 [INFO] panel hidden (focus lost)
2026-10-27 16:34:50 [INFO] panel closed after idle (memory released)
2026-10-27 16:34:50 [INFO] panel destroyed (webview memory released)
```
