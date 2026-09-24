# YouTube Downloader

برنامه دسکتاپ دانلود از یوتیوب برای ویندوز و مک — ساخته‌شده با Tauri 2 (Rust) + React + TypeScript.

A cross-platform (Windows/macOS) YouTube downloader desktop app built with Tauri 2 (Rust) + React + TypeScript.

## امکانات | Features

- **تک ویدیو، پلی‌لیست یا کل کانال** — ابتدا لیست ویدیوها بدون دانلود نمایش داده می‌شود، بعد انتخاب و دانلود
- **شماره‌گذاری خودکار** — پلی‌لیست‌ها به ترتیب خود پلی‌لیست و کانال‌ها از قدیمی‌ترین به جدیدترین: `001 - نام ویدیو.mp4`
- **دانلود منیجر کامل** — صف، دانلود هم‌زمان (قابل تنظیم)، Pause/Resume/Cancel برای هر آیتم، تلاش مجدد، نمایش درصد/سرعت/حجم/زمان باقی‌مانده، تاریخچه
- **فرمت‌ها** — ویدیو MP4 در کیفیت‌های 360p تا 4K، یا استخراج صوت MP3 (با کاور و متادیتا)
- **ساختار پوشه‌ها** — پیش‌فرض: `Downloads/YouTube Downloader/<نام کانال یا پلی‌لیست>/` برای دسته‌ها و `Downloads/YouTube Downloader/Single Videos/` برای تک‌ویدیوها (قابل تغییر)
- **دسته‌بندی در صف دانلود** — آیتم‌های هر کانال/پلی‌لیست زیر یک گروه با شمارنده پیشرفت
- **دوزبانه** — فارسی (RTL) و انگلیسی با تغییر زبان و تم تاریک/روشن
- **ابزارهای خودکار** — yt-dlp و ffmpeg در اولین اجرا دانلود و نگهداری می‌شوند؛ دکمه به‌روزرسانی yt-dlp در تنظیمات
- **زیرنویس** (اختیاری) و **محدودیت سرعت** در تنظیمات
- صف و تنظیمات بین اجراها ذخیره می‌شوند (دانلودهای ناتمام بعد از باز شدن برنامه قابل Resume هستند)

## اجرای توسعه | Development

```bash
npm install
npm run tauri dev
```

پیش‌نیازها: Node.js، Rust toolchain (rustup)، و در ویندوز Visual Studio C++ Build Tools.

## بیلد | Build

```bash
npm run tauri build
```

خروجی‌ها در `src-tauri/target/release/bundle/`:
- `msi/YouTube Downloader_0.1.0_x64_en-US.msi`
- `nsis/YouTube Downloader_0.1.0_x64-setup.exe`

برای بیلد مک، روی macOS همان دستور `npm run tauri build` را اجرا کنید (خروجی .app و .dmg).

## معماری

```
src-tauri/src/
  binaries.rs   دانلود/مدیریت خودکار yt-dlp و ffmpeg
  info.rs       فرمان fetch_info: لیست ویدیوها بدون دانلود (flat-playlist)
  download.rs   مدیر صف، اجرای yt-dlp، پارس پیشرفت، Pause/Resume/Cancel، شماره‌گذاری
  settings.rs   تنظیمات (JSON در AppData)
  history.rs    تاریخچه دانلودها + نمایش فایل در Explorer/Finder
  util.rs       تشخیص نوع URL، نرمال‌سازی URL کانال، پاک‌سازی نام فایل

src/
  App.tsx               تب‌ها، i18n، تم، اوورلی نصب ابزارها
  views/NewDownload.tsx ورودی URL، جدول انتخاب ویدیوها، کیفیت/فرمت/پوشه
  views/Queue.tsx       صف گروه‌بندی‌شده با کنترل‌های هر آیتم
  views/History.tsx     تاریخچه
  views/SettingsView.tsx تنظیمات + وضعیت ابزارها
  i18n.ts               دیکشنری fa/en
```

## نکته

در برخی شبکه‌ها ممکن است yt-dlp نیاز به پراکسی داشته باشد؛ می‌توانید در تنظیمات سیستم عامل پراکسی را تنظیم کنید (yt-dlp از متغیر محیطی `HTTPS_PROXY` استفاده می‌کند).
