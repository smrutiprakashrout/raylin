use std::error::Error;
use std::fs;
use std::path::Path;

use freedesktop_desktop_entry::DesktopEntry;
use nucleo::{Config, Nucleo};
use walkdir::WalkDir;

use crate::models::AppEntry;
use crate::utils::exec_parser::{binary_exists, parse_exec_binary};
use crate::utils::icon_resolver::resolve_icon_path;
use super::Plugin;

// ──────────────────────────────────────────────────────────────────────────────
// AppLauncherPlugin
// ──────────────────────────────────────────────────────────────────────────────

pub struct AppLauncherPlugin {
    apps: Vec<AppEntry>,
    nucleo: Option<Nucleo<AppEntry>>,
}

impl AppLauncherPlugin {
    pub fn new() -> Self {
        Self { apps: Vec::new(), nucleo: None }
    }

    // ── Initialization ───────────────────────────────────────────────────────

    /// Scan all desktop entries and build the nucleo index.
    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Scanning applications...");
        self.apps = self.scan_desktop_entries();

        let mut nucleo = Nucleo::<AppEntry>::new(
            Config::DEFAULT,
            std::sync::Arc::new(|| {}),
            None,
            1,
        );
        let injector = nucleo.injector();
        for app in &self.apps {
            injector.push(app.clone(), |a, c| { c[0] = a.name.clone().into(); });
        }
        while nucleo.tick(10).running {}

        self.nucleo = Some(nucleo);
        Ok(())
    }

    // ── Desktop Scanning ─────────────────────────────────────────────────────

    fn get_desktop_entry_paths(&self) -> Vec<String> {
        let home = std::env::var("HOME").unwrap_or_default();
        vec![
            // User-local first (highest priority)
            format!("{}/.local/share/applications", home),
            format!("{}/.local/share/flatpak/exports/share/applications", home),
            // System-wide
            "/usr/share/applications".to_string(),
            "/usr/local/share/applications".to_string(),
            // Flatpak system
            "/var/lib/flatpak/exports/share/applications".to_string(),
            // Snap
            "/var/lib/snapd/desktop/applications".to_string(),
        ]
    }

    fn scan_desktop_entries(&self) -> Vec<AppEntry> {
        let mut apps: Vec<AppEntry> = Vec::new();

        for search_path in self.get_desktop_entry_paths() {
            if !Path::new(&search_path).exists() { continue; }

            for entry in WalkDir::new(&search_path).into_iter().filter_map(Result::ok) {
                if !entry.path().extension().map_or(false, |e| e == "desktop") { continue; }

                if let Some(app) = self.parse_desktop_entry(entry.path()) {
                    // Deduplication by name — first occurrence wins
                    if !apps.iter().any(|a| a.name.eq_ignore_ascii_case(&app.name)) {
                        apps.push(app);
                    }
                }
            }
        }

        apps
    }

    fn parse_desktop_entry(&self, path: &Path) -> Option<AppEntry> {
        let content = fs::read_to_string(path).ok()?;
        let path_buf = path.to_path_buf();
        let no_locales: Option<&[String]> = None;
        let de = DesktopEntry::from_str(path_buf, &content, no_locales).ok()?;

        // ── Visibility filters ────────────────────────────────────────────────
        if de.no_display() || de.hidden() { return None; }

        // OnlyShowIn: accept if GNOME/KDE/sway/Hyprland/wlroots listed
        let only_show = content.lines()
            .find(|l| l.starts_with("OnlyShowIn="))
            .map(|l| l.trim_start_matches("OnlyShowIn="));
        if let Some(only) = only_show {
            let accepted = ["GNOME", "KDE", "sway", "Hyprland", "wlroots", "X-"];
            if !accepted.iter().any(|de_name| only.contains(de_name)) {
                return None;
            }
        }

        // ── Name ─────────────────────────────────────────────────────────────
        let mut name = String::new();
        for line in content.lines() {
            if line.starts_with("Name=") {
                name = line.trim_start_matches("Name=").to_string();
                break;
            } else if line.starts_with("Name[en_US]=") && name.is_empty() {
                name = line.trim_start_matches("Name[en_US]=").to_string();
            } else if line.starts_with("Name[en]=") && name.is_empty() {
                name = line.trim_start_matches("Name[en]=").to_string();
            }
        }
        if name.is_empty() {
            name = de.name(&[] as &[String]).map(|c| c.to_string()).unwrap_or_default();
        }
        name = name.replace(".desktop", "");
        if name.is_empty() { return None; }

        // ── Exec validation ───────────────────────────────────────────────────
        let exec_raw = de.exec().unwrap_or("").to_string();
        let exec_clean = exec_raw
            .replace("%f", "").replace("%F", "")
            .replace("%u", "").replace("%U", "")
            .replace("%d", "").replace("%D", "")
            .replace("%n", "").replace("%N", "")
            .replace("%i", "").replace("%c", "")
            .replace("%k", "").replace("%v", "").replace("%m", "")
            .trim().to_string();

        if exec_clean.is_empty() { return None; }
        let binary = parse_exec_binary(&exec_clean);
        if binary.is_empty() || !binary_exists(binary) { return None; }

        // ── Terminal / type ───────────────────────────────────────────────────
        let is_terminal = content.lines()
            .find(|l| l.starts_with("Terminal="))
            .map(|l| l.trim_start_matches("Terminal=").trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let type_label = if is_terminal { "Command Line" } else { "Application" }.to_string();

        // ── Package type detection ────────────────────────────────────────────
        let path_str = path.to_string_lossy();
        let package_type = if path_str.contains("flatpak") {
            "Flatpak"
        } else if path_str.contains("snap") {
            "Snap"
        } else {
            "System"
        };

        // Use package type as type_label suffix for snap/flatpak, keep "Application"/"Command Line" for system
        let final_type = if package_type != "System" && !is_terminal {
            package_type.to_string()
        } else {
            type_label
        };

        // ── Icon ──────────────────────────────────────────────────────────────
        let icon_raw = de.icon().unwrap_or("application-x-executable").to_string();
        let icon_path = resolve_icon_path(&icon_raw);

        Some(AppEntry::new(name, exec_clean, icon_path, final_type))
    }

    // ── Search ───────────────────────────────────────────────────────────────

    /// Fuzzy-search apps by name, return up to `limit` results.
    pub fn search(&mut self, query: &str, limit: usize) -> Vec<AppEntry> {
        let nucleo = match &mut self.nucleo {
            Some(n) => n,
            None => return Vec::new(),
        };

        nucleo.pattern.reparse(
            0, query,
            nucleo::pattern::CaseMatching::Ignore,
            nucleo::pattern::Normalization::Smart,
            false,
        );
        while nucleo.tick(10).running {}

        let snapshot = nucleo.snapshot();
        let count = snapshot.matched_item_count().min(limit as u32);
        (0..count)
            .filter_map(|i| snapshot.get_matched_item(i).map(|item| item.data.clone()))
            .collect()
    }

    pub fn get_all_apps(&self) -> &[AppEntry] {
        &self.apps
    }

    // ── Launch ───────────────────────────────────────────────────────────────

    pub fn launch(exec: &str) {
        let mut clean = exec.to_string();
        for code in ["%f", "%F", "%u", "%U", "%c", "%k", "%i", "%m"] {
            clean = clean.replace(code, "");
        }
        clean = clean.trim().to_string();
        if let Err(e) = std::process::Command::new("sh").arg("-c").arg(&clean).spawn() {
            eprintln!("[app_launcher] Failed to spawn {}: {}", clean, e);
        }
    }
}

impl Default for AppLauncherPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// ── Plugin Trait ─────────────────────────────────────────────────────────────

impl Plugin for AppLauncherPlugin {
    fn id(&self) -> &str { "app_launcher" }
    fn name(&self) -> &str { "Application Launcher" }
    fn trigger(&self) -> &str { "" }  // Root mode — no prefix
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn init(&mut self) -> Result<(), Box<dyn Error>> { self.init() }
    fn ui_component(&self) -> &str { "ui/plugin/app_launcher.slint" }
}
