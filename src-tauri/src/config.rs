//! Application configuration: profiles, settings and persistence.
//!
//! The config lives at `%APPDATA%\com.themeswitcher.app\config.json` and is
//! written atomically (temp file + rename) so a crash can never corrupt it.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Identifier of the built-in "Light" profile (also the default Day profile).
pub const LIGHT_PROFILE_ID: &str = "light";
/// Identifier of the built-in "Dark" profile (also the default Night profile).
pub const DARK_PROFILE_ID: &str = "dark";

/// A Windows theme mode value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "light" => Some(ThemeMode::Light),
            "dark" => Some(ThemeMode::Dark),
            _ => None,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        }
    }
}

/// A complete theme profile that can be applied to Windows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeProfile {
    /// Display name shown in the UI (e.g. "Day", "Night").
    pub name: String,
    /// Windows mode: taskbar, Start menu, system surfaces.
    pub windows_mode: ThemeMode,
    /// App mode: default Windows app theme.
    pub app_mode: ThemeMode,
    /// Window transparency effects.
    pub transparency: bool,
    /// Accent color as `#RRGGBB`. `None` = do not change.
    pub accent_color: Option<String>,
    /// Absolute path to a wallpaper image. `None` = do not change.
    pub wallpaper: Option<String>,
}

impl ThemeProfile {
    /// Validate the profile. Wallpaper existence is checked at apply time.
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::new(
                crate::error::ErrorCode::ConfigInvalid,
                "Profile name must not be empty",
            ));
        }
        if let Some(color) = &self.accent_color {
            if !is_valid_hex_color(color) {
                return Err(AppError::new(
                    crate::error::ErrorCode::ConfigInvalid,
                    format!("Invalid accent color '{color}' (expected #RRGGBB)"),
                ));
            }
        }
        if let Some(wallpaper) = &self.wallpaper {
            let path = Path::new(wallpaper);
            if !path.is_absolute() {
                return Err(AppError::new(
                    crate::error::ErrorCode::ConfigInvalid,
                    "Wallpaper path must be absolute",
                ));
            }
        }
        Ok(())
    }
}

/// Check whether `s` is a `#RRGGBB` hex color.
pub fn is_valid_hex_color(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 7 || bytes[0] != b'#' {
        return false;
    }
    s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Application-wide settings (deliberately minimal - the app is a tray popup).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub start_with_windows: bool,
    pub check_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            start_with_windows: false,
            check_updates: true,
        }
    }
}

/// Root configuration document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    pub settings: Settings,
    pub profiles: HashMap<String, ThemeProfile>,
    pub last_active_profile: Option<String>,
}

pub const CONFIG_VERSION: u32 = 1;

impl Default for AppConfig {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(
            LIGHT_PROFILE_ID.to_string(),
            ThemeProfile {
                name: "Day".to_string(),
                windows_mode: ThemeMode::Light,
                app_mode: ThemeMode::Light,
                transparency: true,
                accent_color: None,
                wallpaper: None,
            },
        );
        profiles.insert(
            DARK_PROFILE_ID.to_string(),
            ThemeProfile {
                name: "Night".to_string(),
                windows_mode: ThemeMode::Dark,
                app_mode: ThemeMode::Dark,
                transparency: true,
                accent_color: None,
                wallpaper: None,
            },
        );
        Self {
            version: CONFIG_VERSION,
            settings: Settings::default(),
            profiles,
            last_active_profile: None,
        }
    }
}

impl AppConfig {
    /// Load the config from `path`. A missing or invalid file falls back to
    /// defaults (invalid files are backed up so the user does not lose data).
    pub fn load(path: &Path) -> AppResult<Self> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let config = Self::default();
                config.save(path)?;
                return Ok(config);
            }
            Err(e) => return Err(AppError::new(crate::error::ErrorCode::ConfigRead, e.to_string())),
        };

        match serde_json::from_str::<AppConfig>(&content) {
            Ok(mut config) => {
                config.migrate_and_fill();
                config.validate()?;
                Ok(config)
            }
            Err(e) => {
                // Preserve the broken file for recovery, then start fresh.
                let backup = path.with_extension("json.corrupt");
                let _ = fs::copy(path, &backup);
                crate::logger::warn(format!("config invalid ({e}), backed up to {backup:?}"));
                let config = Self::default();
                config.save(path)?;
                Ok(config)
            }
        }
    }

    /// Ensure required invariants after deserialization (forward/backward
    /// compatibility with older config files).
    fn migrate_and_fill(&mut self) {
        self.version = CONFIG_VERSION;
        self.profiles
            .entry(LIGHT_PROFILE_ID.to_string())
            .or_insert_with(|| Self::default().profiles[LIGHT_PROFILE_ID].clone());
        self.profiles
            .entry(DARK_PROFILE_ID.to_string())
            .or_insert_with(|| Self::default().profiles[DARK_PROFILE_ID].clone());
    }

    /// Validate all profiles and settings.
    pub fn validate(&self) -> AppResult<()> {
        for (id, profile) in &self.profiles {
            profile
                .validate()
                .map_err(|e| AppError::new(e.code, format!("profile '{id}': {}", e.message)))?;
        }
        Ok(())
    }

    /// Persist the config atomically.
    pub fn save(&self, path: &Path) -> AppResult<()> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::new(crate::error::ErrorCode::ConfigWrite, e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::new(crate::error::ErrorCode::ConfigWrite, e.to_string()))?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json.as_bytes())
            .map_err(|e| AppError::new(crate::error::ErrorCode::ConfigWrite, e.to_string()))?;
        fs::rename(&tmp, path)
            .map_err(|e| AppError::new(crate::error::ErrorCode::ConfigWrite, e.to_string()))?;
        Ok(())
    }

    /// Get a profile by id.
    pub fn profile(&self, id: &str) -> Option<&ThemeProfile> {
        self.profiles.get(id)
    }

    /// Get a mutable profile by id.
    pub fn profile_mut(&mut self, id: &str) -> Option<&mut ThemeProfile> {
        self.profiles.get_mut(id)
    }

    /// Resolve the config file path for a given app config directory.
    pub fn config_path(app_config_dir: &Path) -> PathBuf {
        app_config_dir.join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_both_primary_profiles() {
        let config = AppConfig::default();
        assert!(config.profile(LIGHT_PROFILE_ID).is_some());
        assert!(config.profile(DARK_PROFILE_ID).is_some());
        assert_eq!(
            config.profile(DARK_PROFILE_ID).unwrap().app_mode,
            ThemeMode::Dark
        );
        assert_eq!(config.version, CONFIG_VERSION);
    }

    #[test]
    fn hex_color_validation() {
        assert!(is_valid_hex_color("#0078D4"));
        assert!(is_valid_hex_color("#000000"));
        assert!(is_valid_hex_color("#ffffff"));
        assert!(!is_valid_hex_color("0078D4"));
        assert!(!is_valid_hex_color("#0078D"));
        assert!(!is_valid_hex_color("#0078D44"));
        assert!(!is_valid_hex_color("#GGGGGG"));
        assert!(!is_valid_hex_color(""));
    }

    #[test]
    fn profile_validation_rejects_bad_values() {
        let mut p = ThemeProfile {
            name: "X".into(),
            windows_mode: ThemeMode::Light,
            app_mode: ThemeMode::Light,
            transparency: true,
            accent_color: Some("not-a-color".into()),
            wallpaper: None,
        };
        assert!(p.validate().is_err());
        p.accent_color = Some("#AABBCC".into());
        assert!(p.validate().is_ok());
        p.name = "  ".into();
        assert!(p.validate().is_err());
    }

    #[test]
    fn roundtrip_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn camel_case_json_field_names() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"startWithWindows\""));
        assert!(json.contains("\"windowsMode\""));
        assert!(json.contains("\"lastActiveProfile\""));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("wts_config_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("config.json");
        let _ = fs::remove_file(&path);

        let mut config = AppConfig::default();
        config.settings.check_updates = false;
        config.settings.start_with_windows = true;
        config.save(&path).unwrap();

        let loaded = AppConfig::load(&path).unwrap();
        assert!(!loaded.settings.check_updates);
        assert!(loaded.settings.start_with_windows);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_missing_creates_default() {
        let dir = std::env::temp_dir().join("wts_config_test_missing");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("does_not_exist.json");
        let _ = fs::remove_file(&path);
        let loaded = AppConfig::load(&path).unwrap();
        assert_eq!(loaded, AppConfig::default());
        assert!(path.exists());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_invalid_backs_up_and_resets() {
        let dir = std::env::temp_dir().join("wts_config_test_invalid");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("config.json");
        fs::write(&path, "{ this is not valid json ").unwrap();
        let loaded = AppConfig::load(&path).unwrap();
        assert_eq!(loaded, AppConfig::default());
        assert!(path.with_extension("json.corrupt").exists());
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("json.corrupt"));
    }

    #[test]
    fn theme_mode_opposites() {
        assert_eq!(ThemeMode::Light.opposite(), ThemeMode::Dark);
        assert_eq!(ThemeMode::Dark.opposite(), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str_opt("light"), Some(ThemeMode::Light));
        assert_eq!(ThemeMode::from_str_opt("bogus"), None);
    }
}
