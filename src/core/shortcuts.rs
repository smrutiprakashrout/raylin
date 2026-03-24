// ─────────────────────────────────────────────────────────────────────────────
// shortcuts.rs — Single source of truth for ALL mode triggers and aliases
//
// To change a trigger or add a new alias, edit ONLY this file.
// ─────────────────────────────────────────────────────────────────────────────

// ── Mode triggers (typed in the search bar from Root mode) ───────────────────

/// Emoji picker
pub const TRIGGER_EMOJI: &str = "@e";

/// Clipboard history
pub const TRIGGER_CLIPBOARD: &str = "@c";

/// System actions (Lock, Sleep, Restart, Suspend, Power Off)
pub const TRIGGER_SYSTEM: &str = "@s";

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns `true` if `query` exactly matches or starts with `"{trigger} "`.
pub fn matches_trigger(query: &str, trigger: &str) -> bool {
    query == trigger || query.starts_with(&format!("{trigger} "))
}
