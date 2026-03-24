use std::path::PathBuf;
use walkdir::WalkDir;

/// Freedesktop-compliant icon lookup.
///
/// Search strategy (first match wins):
///   1. Absolute path  — used directly if it exists
///   2. Priority dirs  — specific size/scalable dirs checked without walking (fast path)
///   3. Theme walk     — WalkDir over icon roots, filtered to `apps/` subdirs
///   4. Snap           — /snap/<name>/current/meta/gui/icon.*
///   5. Reverse-DNS    — Flatpak: org.app.Name → Name (last component)
///
/// Extension priority: svg → png → jpg → ico → xpm
pub fn find_icon_path(icon_name: &str) -> Option<PathBuf> {
    if icon_name.is_empty() { return None; }

    // ── 1. Absolute path ────────────────────────────────────────────────────
    if icon_name.starts_with('/') {
        let p = PathBuf::from(icon_name);
        return if p.exists() { Some(p) } else { None };
    }

    let home = std::env::var("HOME").unwrap_or_default();

    // Extension search order — SVG first (vector, scales perfectly), xpm last
    let exts: &[&str] = &[".svg", ".png", ".jpg", ".ico", ".xpm", ""];

    // ── 2. Priority dirs (fast path — covers 95% of real-world installs) ────
    let priority_dirs: Vec<PathBuf> = vec![
        // User local theme
        PathBuf::from(format!("{}/.local/share/icons/hicolor/scalable/apps", home)),
        PathBuf::from(format!("{}/.local/share/icons/hicolor/256x256/apps",  home)),
        PathBuf::from(format!("{}/.local/share/icons/hicolor/128x128/apps",  home)),
        PathBuf::from(format!("{}/.local/share/icons/hicolor/64x64/apps",    home)),
        PathBuf::from(format!("{}/.local/share/icons/hicolor/48x48/apps",    home)),
        PathBuf::from(format!("{}/.local/share/pixmaps",                     home)),
        // System hicolor
        PathBuf::from("/usr/share/icons/hicolor/scalable/apps"),
        PathBuf::from("/usr/share/icons/hicolor/256x256/apps"),
        PathBuf::from("/usr/share/icons/hicolor/128x128/apps"),
        PathBuf::from("/usr/share/icons/hicolor/64x64/apps"),
        PathBuf::from("/usr/share/icons/hicolor/48x48/apps"),
        PathBuf::from("/usr/share/icons/hicolor/32x32/apps"),
        PathBuf::from("/usr/share/pixmaps"),
        // GNOME Adwaita
        PathBuf::from("/usr/share/icons/Adwaita/scalable/apps"),
        PathBuf::from("/usr/share/icons/Adwaita/256x256/apps"),
        // KDE Breeze
        PathBuf::from("/usr/share/icons/breeze/apps/48"),
        PathBuf::from("/usr/share/icons/breeze-dark/apps/48"),
        // Flatpak system exports
        PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor/scalable/apps"),
        PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor/256x256/apps"),
        PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor/128x128/apps"),
        PathBuf::from("/var/lib/flatpak/exports/share/icons/hicolor/48x48/apps"),
        // Flatpak user exports
        PathBuf::from(format!("{}/.local/share/flatpak/exports/share/icons/hicolor/scalable/apps", home)),
        PathBuf::from(format!("{}/.local/share/flatpak/exports/share/icons/hicolor/256x256/apps", home)),
        PathBuf::from(format!("{}/.local/share/flatpak/exports/share/icons/hicolor/128x128/apps", home)),
    ];

    for dir in &priority_dirs {
        if !dir.exists() { continue; }
        for ext in exts {
            let candidate = dir.join(format!("{}{}", icon_name, ext));
            if candidate.exists() { return Some(candidate); }
        }
    }

    // ── 3. Theme walk — catches arbitrary themes and non-standard sizes ──────
    let walk_roots: Vec<PathBuf> = vec![
        PathBuf::from(format!("{}/.local/share/icons", home)),
        PathBuf::from("/usr/share/icons"),
        PathBuf::from(format!("{}/.local/share/flatpak/exports/share/icons", home)),
        PathBuf::from("/var/lib/flatpak/exports/share/icons"),
    ];

    for root in &walk_roots {
        if !root.exists() { continue; }
        for entry in WalkDir::new(root)
            .min_depth(3)
            .max_depth(5)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
            .filter(|e| e.path().components().any(|c| c.as_os_str() == "apps"))
        {
            for ext in exts {
                let candidate = entry.path().join(format!("{}{}", icon_name, ext));
                if candidate.exists() { return Some(candidate); }
            }
        }
    }

    // ── 4. Snap ─────────────────────────────────────────────────────────────
    for ext in [".png", ".svg"] {
        let p = PathBuf::from(format!("/snap/{}/current/meta/gui/icon{}", icon_name, ext));
        if p.exists() { return Some(p); }
    }

    // ── 5. Reverse-DNS alias (Flatpak: org.localsend.localsend_app → localsend_app) ──
    if icon_name.contains('.') {
        if let Some(short_name) = icon_name.split('.').last() {
            if short_name != icon_name {
                return find_icon_path(short_name);
            }
        }
    }

    None
}

/// Thin wrapper: returns an empty String when no icon is found.
pub fn resolve_icon_path(icon_name: &str) -> String {
    find_icon_path(icon_name)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name_returns_none() {
        assert!(find_icon_path("").is_none());
    }

    #[test]
    fn absolute_path_nonexistent_returns_none() {
        assert!(find_icon_path("/definitely/does/not/exist.svg").is_none());
    }

    #[test]
    fn absolute_path_existing_returns_some() {
        // Use /etc/hostname as a known-existing file
        let result = find_icon_path("/etc/hostname");
        // It doesn't start with '/' in icon_name semantics... actually it does.
        // The file exists, so we get Some.
        assert!(result.is_some());
    }

    #[test]
    fn reverse_dns_no_panic() {
        // Should not panic even for deeply nested reverse-DNS names
        let _ = find_icon_path("org.gnome.Calculator");
    }

    #[test]
    fn resolve_icon_path_empty_for_missing() {
        let result = resolve_icon_path("_this_icon_does_not_exist_xyz_");
        assert!(result.is_empty());
    }
}
