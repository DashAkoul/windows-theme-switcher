//! System tray icon with a dynamic sun/moon icon and a compact context menu.
//!
//! UX:
//! - Left click  -> instantly toggle between the Light and Dark profiles.
//! - Right click -> small menu: switch / profiles / open panel / autostart / exit.
//! - The icon shows what clicking will DO: moon while Windows is Light
//!   (click -> Dark), sun while Windows is Dark (click -> Light).

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::config::{AppConfig, DARK_PROFILE_ID, LIGHT_PROFILE_ID};
use crate::theme::ThemeState;
use crate::AppState;

/// Moon icon - shown while Windows is in Light mode (click switches to Dark).
const ICON_MOON: &[u8] = include_bytes!("../icons/tray-moon.png");
/// Sun icon - shown while Windows is in Dark mode (click switches to Light).
const ICON_SUN: &[u8] = include_bytes!("../icons/tray-sun.png");

fn icon_for(state: &ThemeState) -> Image<'static> {
    let bytes: &'static [u8] = if state.app_mode_light { ICON_MOON } else { ICON_SUN };
    Image::from_bytes(bytes).expect("tray icons are valid PNGs compiled into the binary")
}

fn tooltip_for(state: &ThemeState) -> String {
    if state.app_mode_light {
        "Windows Theme Switcher — Light (click for Dark)".to_string()
    } else {
        "Windows Theme Switcher — Dark (click for Light)".to_string()
    }
}

fn profile_name(config: &AppConfig, id: &str, fallback: &str) -> String {
    config
        .profile(id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| fallback.to_string())
}

/// Build the context menu for the current state.
fn build_menu(
    app: &AppHandle,
    config: &AppConfig,
    state: &ThemeState,
) -> tauri::Result<Menu<tauri::Wry>> {
    let light_name = profile_name(config, LIGHT_PROFILE_ID, "Light");
    let dark_name = profile_name(config, DARK_PROFILE_ID, "Dark");

    let switch_label = if state.app_mode_light {
        format!("Switch to {dark_name}")
    } else {
        format!("Switch to {light_name}")
    };

    let switch = MenuItem::with_id(app, "switch", switch_label, true, None::<&str>)?;
    let light = CheckMenuItem::with_id(
        app,
        "light",
        format!("{light_name} (Light)"),
        true,
        state.app_mode_light,
        None::<&str>,
    )?;
    let dark = CheckMenuItem::with_id(
        app,
        "dark",
        format!("{dark_name} (Dark)"),
        true,
        !state.app_mode_light,
        None::<&str>,
    )?;
    let panel = MenuItem::with_id(app, "panel", "Open panel…", true, None::<&str>)?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        "Start with Windows",
        true,
        config.settings.start_with_windows,
        None::<&str>,
    )?;
    let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;

    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    Menu::with_items(
        app,
        &[&switch, &sep1, &light, &dark, &sep2, &panel, &autostart, &sep3, &exit],
    )
}

/// Create the tray icon and register all event handlers.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let config = state.config.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let theme = crate::theme::read_theme_state().unwrap_or_else(|_| default_state());

    let menu = build_menu(app, &config, &theme)?;
    let icon = icon_for(&theme);
    let tooltip = tooltip_for(&theme);

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip(tooltip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Err(e) = crate::commands::toggle_theme_inner(tray.app_handle()) {
                    crate::logger::error(format!("tray toggle failed: {e}"));
                    let _ = tray.app_handle().emit("theme-error", e.message.clone());
                }
                // The toggle rebuilds the menu and swaps the icon; release
                // those pages once the broadcast has finished.
                crate::memory::trim_soon(3500);
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
        "panel" => crate::show_panel(app),
        "autostart" => {
            let _ = crate::commands::toggle_autostart_inner(app);
        }
        "exit" => {
            crate::logger::info("exit requested from tray");
            app.exit(0);
        }
        _ => {}
    }

    // Menu handling allocates (menu rebuild, icon swap, registry writes).
    // Give the pages back once the work has settled, so the process goes
    // back to a few hundred KB while it waits for the next click.
    if id != "exit" && id != "panel" {
        crate::memory::trim_soon(2500);
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
    let _ = tray.set_tooltip(Some(tooltip_for(state)));
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
        assert!(Image::from_bytes(ICON_MOON).is_ok());
        assert!(Image::from_bytes(ICON_SUN).is_ok());
    }

    #[test]
    fn tooltip_mentions_next_action() {
        let light = ThemeState {
            app_mode_light: true,
            system_mode_light: true,
            transparency: true,
            accent_color: None,
            wallpaper: None,
        };
        assert!(tooltip_for(&light).contains("click for Dark"));

        let dark = ThemeState {
            app_mode_light: false,
            system_mode_light: false,
            transparency: true,
            accent_color: None,
            wallpaper: None,
        };
        assert!(tooltip_for(&dark).contains("click for Light"));
    }

    #[test]
    fn default_state_is_dark() {
        assert!(!default_state().app_mode_light);
    }
}
