use serde::{Deserialize, Serialize};

/// A single installable application discovered from .desktop files.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon_path: String,
    /// "Application" or "Command Line"
    pub type_label: String,
}

impl AppEntry {
    pub fn new(name: impl Into<String>, exec: impl Into<String>, icon_path: impl Into<String>, type_label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            exec: exec.into(),
            icon_path: icon_path.into(),
            type_label: type_label.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_entry_new() {
        let entry = AppEntry::new("Firefox", "firefox %u", "/usr/share/icons/firefox.svg", "Application");
        assert_eq!(entry.name, "Firefox");
        assert_eq!(entry.type_label, "Application");
    }

    #[test]
    fn app_entry_clone() {
        let entry = AppEntry::new("Test", "test", "", "Application");
        let cloned = entry.clone();
        assert_eq!(cloned.name, entry.name);
    }
}
