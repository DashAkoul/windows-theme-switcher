//! System tray icon with dynamic sun/moon icon and context menu.
//!
//! UX contract:
//! - Left click  -> instantly toggle between the Light and Dark profiles.
//! - Right click -> context menu with switch/current/profiles/settings/
//!   autostart/about/exit.
//! - The icon shows what clicking will DO: moon while Windows is Light
//!   (click -> Dark), sun while Windows is Dark (click -> Light).

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::config::{AppConfig, Language, DARK_PROFILE_ID, LIGHT_PROFILE_ID};
use crate::theme::ThemeState;
use crate::AppState;

/// Moon icon - shown while Windows is in Light mode (click switches to Dark).
const ICON_MOON: &[u8] = include_bytes!("../icons/tray-moon.png");
/// Sun icon - shown while Windows is in Dark mode (click switches to Light).
const ICON_SUN: &[u8] = include_bytes!("../icons/tray-sun.png");

/// Tiny i18n helper for the tray menu (kept in sync with the frontend).
fn tr(lang: Language, key: &str) -> String {
    let (en, fa): (&str, &str) = match key {
        "switch_dark" => ("Switch to Dark", "تغییر به تیره"),
        "switch_light" => ("Switch to Light", "تغییر به روشن"),
        "current" => ("Current", "وضعیت فعلی"),
        "light" => ("Light", "روشن"),
        "dark" => ("Dark", "تیره"),
        "settings" => ("Settings", "تنظیمات"),
        "autostart" => ("Start with Windows", "اجرا با ویندوز"),
        "about" => ("About", "درباره"),
        "exit" => ("Exit", "خروج"),
        _ => (key, key),
    };
    match lang {
        Language::En => en.to_string(),
        Language::Fa => fa.to_string(),
    }
}

fn icon_for(state: &ThemeState) -> Image<'static> {
    // Light mode -> moon (click goes to dark); Dark mode -> sun.
    let bytes: &'static [u8] = if state.app_mode_light { ICON_MOON } else { ICON_SUN };
    Image::from_bytes(bytes).expect("tray icons are valid PNGs compiled into the binary")
}

fn tooltip_for(config: &AppConfig, state: &ThemeState) -> String {
    let lang = config.settings.language;
    let current = if state.app_mode_light {
        tr(lang, "light")
    } else {
        tr(lang, "dark")
    };
    let next = if state.app_mode_light {
        tr(lang, "switch_dark")
    } else {
        tr(lang, "switch_light")
    };
    format!("Windows Theme Switcher\n{current} · {next}")
}

/// Build the context menu for the current state.
fn build_menu(
    app: &AppHandle,
    config: &AppConfig,
    state: &ThemeState,
) -> tauri::Result<Menu<tauri::Wry>> {
    let lang = config.settings.language;
    let light_name = config
        .profile(LIGHT_PROFILE_ID)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| tr(lang, "light"));
    let dark_name = config
        .profile(DARK_PROFILE_ID)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| tr(lang, "dark"));

    let current_label = if state.app_mode_light {
        format!("● {} · {}", tr(lang, "light"), light_name)
    } else {
        format!("● {} · {}", tr(lang, "dark"), dark_name)
    };

    let switch_label = if state.app_mode_light {
        format!("{} ({})", tr(lang, "switch_dark"), dark_name)
    } else {
        format!("{} ({})", tr(lang, "switch_light"), light_name)
    };

    let switch = MenuItem::with_id(app, "switch", switch_label, true, None::<&str>)?;
    let current = MenuItem::with_id(app, "current", current_label, false, None::<&str>)?;
    let light = CheckMenuItem::with_id(
        app,
        "light",
        format!("{} ({})", tr(lang, "light"), light_name),
        true,
        state.app_mode_light,
        None::<&str>,
    )?;
    let dark = CheckMenuItem::with_id(
        app,
        "dark",
        format!("{} ({})", tr(lang, "dark"), dark_name),
        true,
        !state.app_mode_light,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, "settings", tr(lang, "settings"), true, None::<&str>)?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        tr(lang, "autostart"),
        true,
        config.settings.start_with_windows,
        None::<&str>,
    )?;
    let about = MenuItem::with_id(app, "about", tr(lang, "about"), true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", tr(lang, "exit"), true, None::<&str>)?;

    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    Menu::with_items(
        app,
        &[
            &switch,
            &current,
            &sep1,
            &light,
            &dark,
            &sep2,
            &settings,
            &autostart,
            &sep3,
            &about,
            &exit,
        ],
    )
}

/// Create the tray icon and register all event handlers.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let config = state.config.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let theme = crate::theme::read_theme_state().unwrap_or_else(|_| default_state());

    let menu = build_menu(app, &config, &theme)?;
    let icon = icon_for(&theme);
    let tooltip = tooltip_for(&config, &theme);

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip(tooltip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // Fast path: toggle theme directly from the tray.
                if let Err(e) = crate::commands::toggle_theme_inner(_tray.app_handle()) {
                    crate::logger::error(format!("tray toggle failed: {e}"));
                    let _ = _tray.app_handle().emit("theme-error", e.message.clone());
                }
            }
        })
        .build(app)?;

    *app
        .state::<AppState>()
        .tray
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = Some(tray);

    Ok(())
}

/// Handle a context-menu click.
fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "switch" => {
            if let Err(e) = crate::commands::toggle_theme_inner(app) {
                crate::logger::error(format!("menu toggle failed: {e}"));
                let _ = app.emit("theme-error", e.message.clone());
            }
        }
        "light" => {
            let _ = crate::commands::apply_profile_inner(app, LIGHT_PROFILE_ID);
        }
        "dark" => {
            let _ = crate::commands::apply_profile_inner(app, DARK_PROFILE_ID);
        }
        "settings" => {
            crate::commands::show_window_with_route(app, "settings");
        }
        "about" => {
            crate::commands::show_window_with_route(app, "about");
        }
        "autostart" => {
            let _ = crate::commands::toggle_autostart_inner(app);
        }
        "exit" => {
            crate::logger::info("exit requested from tray");
            app.exit(0);
        }
        _ => {}
    }
}

/// Refresh icon, tooltip and menu after a theme change.
pub fn refresh(app: &AppHandle, state: &ThemeState) {
    let app_state = app.state::<AppState>();

    let config = match app_state.config.lock() {
        Ok(c) => c.clone(),
        Err(e) => e.into_inner().clone(),
    };

    let tray = app_state.tray.lock().unwrap_or_else(|e| e.into_inner());
    let Some(tray) = tray.as_ref() else {
        return;
    };

    if let Err(e) = tray.set_icon(Some(icon_for(state))) {
        crate::logger::warn(format!("failed to update tray icon: {e}"));
    }
    let _ = tray.set_tooltip(Some(tooltip_for(&config, state)));
    if let Ok(menu) = build_menu(app, &config, state) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// Fallback state if the registry read fails at startup (assume dark).
fn default_state() -> ThemeState {
    ThemeState {
        app_mode_light: false,
        system_mode_light: false,
        transparency: true,
        accent_color: None,
        wallpaper: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_are_valid_pngs() {
        // The bytes are compiled in; they must decode as PNGs.
        assert!(Image::from_bytes(ICON_MOON).is_ok());
        assert!(Image::from_bytes(ICON_SUN).is_ok());
    }

    #[test]
    fn tooltip_mentions_switch() {
        let config = AppConfig::default();
        let state = ThemeState {
            app_mode_light: true,
            system_mode_light: true,
            transparency: true,
            accent_color: None,
            wallpaper: None,
        };
        let tip = tooltip_for(&config, &state);
        assert!(tip.contains("Windows Theme Switcher"));
        assert!(tip.to_lowercase().contains("dark"));
    }

    #[test]
    fn tr_returns_both_languages() {
        assert_eq!(tr(Language::En, "exit"), "Exit");
        assert_eq!(tr(Language::Fa, "exit"), "خروج");
        assert_eq!(tr(Language::En, "settings"), "Settings");
    }

    #[test]
    fn default_state_is_dark() {
        assert!(!default_state().app_mode_light);
        assert_eq!(crate::config::ThemeMode::Light.as_str(), "light");
    }
}
