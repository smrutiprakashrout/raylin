use std::error::Error;

// ──────────────────────────────────────────────────────────────────────────────
// Plugin Trait
// ──────────────────────────────────────────────────────────────────────────────

/// Every feature plugin must implement this trait.
pub trait Plugin: Send + Sync {
    /// Unique machine-readable identifier (e.g. "app_launcher")
    fn id(&self) -> &str;

    /// Human-readable display name (e.g. "App Launcher")
    fn name(&self) -> &str;

    /// Primary text trigger (e.g. ":a") — empty string means always-on default
    fn trigger(&self) -> &str;

    /// Additional aliases that also activate this plugin
    fn aliases(&self) -> Vec<&str>;

    /// Called once at startup for initialization (DB setup, file scanning, etc.)
    fn init(&mut self) -> Result<(), Box<dyn Error>>;

    /// Path to the .slint component file for this plugin's UI
    fn ui_component(&self) -> &str;
}

// ──────────────────────────────────────────────────────────────────────────────
// Plugin Registry
// ──────────────────────────────────────────────────────────────────────────────

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn get_by_id(&self, id: &str) -> Option<&dyn Plugin> {
        self.plugins.iter().find(|p| p.id() == id).map(|p| p.as_ref())
    }

    pub fn get_by_trigger(&self, trigger: &str) -> Option<&dyn Plugin> {
        self.plugins.iter().find(|p| {
            p.trigger() == trigger || p.aliases().contains(&trigger)
        }).map(|p| p.as_ref())
    }

    pub fn all(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }

    pub fn init_all(&mut self) -> Vec<(String, Box<dyn Error>)> {
        let mut errors = Vec::new();
        for plugin in &mut self.plugins {
            if let Err(e) = plugin.init() {
                errors.push((plugin.id().to_string(), e));
            }
        }
        errors
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Plugin stub module declarations
// ──────────────────────────────────────────────────────────────────────────────

pub mod app_launcher;
pub mod emoji;
pub mod clipboard_history;
pub mod systemactions;
