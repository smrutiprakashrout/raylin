use std::error::Error;
use std::fmt;
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────────────
// Config Structures
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub first_run: bool,
    pub theme: String,
    /// "auto" | "gnome" | "kde" | "sway" | "hyprland" | "x11"
    pub desktop_environment: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            first_run: true,
            theme: "catppuccin-mocha".to_string(),
            desktop_environment: "auto".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalShortcuts {
    pub main_launcher: String,
    pub emoji_picker: String,
    pub clipboard_history: String,
}

impl Default for GlobalShortcuts {
    fn default() -> Self {
        Self {
            main_launcher: "Super+Space".to_string(),
            emoji_picker: "Super+E".to_string(),
            clipboard_history: "Super+V".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppShortcuts {
    pub next_item: String,
    pub prev_item: String,
    pub execute: String,
    pub cancel: String,
}

impl Default for InAppShortcuts {
    fn default() -> Self {
        Self {
            next_item: "Down".to_string(),
            prev_item: "Up".to_string(),
            execute: "Enter".to_string(),
            cancel: "Escape".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    pub global: GlobalShortcuts,
    pub in_app: InAppShortcuts,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            global: GlobalShortcuts::default(),
            in_app: InAppShortcuts::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginsConfig {
    pub app_launcher: PluginConfig,
    pub emoji: PluginConfig,
    pub clipboard: PluginConfig,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            app_launcher: PluginConfig::default(),
            emoji: PluginConfig::default(),
            clipboard: PluginConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub shortcuts: ShortcutsConfig,
    pub plugins: PluginsConfig,
}

// ──────────────────────────────────────────────────────────────────────────────
// Desktop Environment Detection
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Sway,
    Hyprland,
    Wlroots,
    X11,
    Unknown,
}

impl fmt::Display for DesktopEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DesktopEnvironment::Gnome => write!(f, "gnome"),
            DesktopEnvironment::Kde => write!(f, "kde"),
            DesktopEnvironment::Sway => write!(f, "sway"),
            DesktopEnvironment::Hyprland => write!(f, "hyprland"),
            DesktopEnvironment::Wlroots => write!(f, "wlroots"),
            DesktopEnvironment::X11 => write!(f, "x11"),
            DesktopEnvironment::Unknown => write!(f, "unknown"),
        }
    }
}

/// Detect the current desktop environment from environment variables.
pub fn detect_desktop() -> DesktopEnvironment {
    // Check Wayland backend first
    let xdg = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();
    let session = std::env::var("DESKTOP_SESSION")
        .unwrap_or_default()
        .to_lowercase();

    if xdg.contains("gnome") || session.contains("gnome") {
        return DesktopEnvironment::Gnome;
    }
    if xdg.contains("kde") || xdg.contains("plasma") || session.contains("plasma") {
        return DesktopEnvironment::Kde;
    }
    if xdg.contains("sway") || session.contains("sway") {
        return DesktopEnvironment::Sway;
    }
    if xdg.contains("hyprland") || session.contains("hyprland") {
        return DesktopEnvironment::Hyprland;
    }

    // X11 fallback
    if std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_ok() {
        return DesktopEnvironment::X11;
    }

    DesktopEnvironment::Unknown
}

// ──────────────────────────────────────────────────────────────────────────────
// Load / Save
// ──────────────────────────────────────────────────────────────────────────────

impl AppConfig {
    /// Load config from `~/.config/wayland-palette/config.toml`.
    /// Falls back to defaults if the file is missing or malformed.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<AppConfig>(&content) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("[config] Failed to parse config: {e}. Using defaults."),
                },
                Err(e) => eprintln!("[config] Failed to read config: {e}. Using defaults."),
            }
        }

        // First run — write defaults to disk
        let default_cfg = AppConfig::default();
        if let Err(e) = default_cfg.save() {
            eprintln!("[config] Failed to write default config: {e}");
        }
        default_cfg
    }

    /// Persist the current config to disk.
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Returns `~/.config/wayland-palette/config.toml`
    fn config_path() -> std::path::PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(format!("{}/.config/wayland-palette/config.toml", home))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.general.theme, "catppuccin-mocha");
        assert!(cfg.plugins.app_launcher.enabled);
        assert_eq!(cfg.shortcuts.global.main_launcher, "Super+Space");
    }

    #[test]
    fn detect_desktop_returns_something() {
        // Just ensure it doesn't panic
        let _ = detect_desktop();
    }

    #[test]
    fn config_roundtrip() {
        let original = AppConfig::default();
        let serialized = toml::to_string_pretty(&original).unwrap();
        let parsed: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.general.theme, original.general.theme);
        assert_eq!(parsed.general.first_run, original.general.first_run);
    }
}
