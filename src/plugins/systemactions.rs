use std::error::Error;
use crate::plugins::Plugin;
use std::process::Command;

pub struct SystemActionsPlugin {
}

impl SystemActionsPlugin {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute_action(&self, action: &str) {
        println!("[system] Action: {}", action);
        let cmd: &str = match action {
            "logout"    => "loginctl terminate-session ${XDG_SESSION_ID}",
            "togglednd" => "if command -v swaync-client >/dev/null 2>&1; then swaync-client -d; elif command -v makoctl >/dev/null 2>&1; then makoctl mode -t dnd; elif command -v gsettings >/dev/null 2>&1; then if [ \"$(gsettings get org.gnome.desktop.notifications show-banners)\" = \"true\" ]; then gsettings set org.gnome.desktop.notifications show-banners false; else gsettings set org.gnome.desktop.notifications show-banners true; fi; fi",
            "restart"   => "systemctl reboot",
            "suspend"   => "systemctl suspend",
            "poweroff"  => "systemctl poweroff",
            _           => return,
        };
        if let Err(e) = Command::new("sh").arg("-c").arg(cmd).spawn() {
            eprintln!("[system] Failed to run '{}': {}", cmd, e);
        }
        std::process::exit(0);
    }
}

impl Plugin for SystemActionsPlugin {
    fn id(&self) -> &str {
        "system_actions"
    }

    fn name(&self) -> &str {
        "System Actions"
    }

    fn trigger(&self) -> &str {
        "@s"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["sys", "power"]
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn ui_component(&self) -> &str {
        "ui/plugin/systemactions.slint"
    }
}
