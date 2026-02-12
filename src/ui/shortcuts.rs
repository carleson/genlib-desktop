use egui;

use crate::models::config::{KeyboardShortcut, ShortcutAction, ShortcutMap, ShortcutModifiers};

/// Hanterar kortkommandon och matchar tangentinmatning mot konfigurerade genvägar
pub struct ShortcutManager {
    shortcuts: ShortcutMap,
}

impl ShortcutManager {
    pub fn new(shortcuts: ShortcutMap) -> Self {
        Self { shortcuts }
    }

    /// Uppdatera genvägar (efter ändring i inställningar)
    pub fn update_shortcuts(&mut self, shortcuts: ShortcutMap) {
        self.shortcuts = shortcuts;
    }

    /// Kolla alla genvägar mot aktuell input.
    /// Returnerar None om ingen matchar eller om textfält har fokus.
    /// `capturing` — true om inställningsvyn fångar en ny genväg.
    pub fn check(&self, ctx: &egui::Context, capturing: bool) -> Option<ShortcutAction> {
        if capturing {
            return None;
        }

        let text_focused = ctx.wants_keyboard_input();

        ctx.input(|input| {
            for (action, shortcut) in &self.shortcuts {
                if !input.key_pressed(shortcut.key) {
                    continue;
                }

                if !shortcut.matches(shortcut.key, &input.modifiers) {
                    continue;
                }

                // Om textfält har fokus: tillåt bara Escape (CloseModal)
                if text_focused {
                    if *action == ShortcutAction::CloseModal
                        && shortcut.key == egui::Key::Escape
                        && !shortcut.modifiers.ctrl_or_cmd
                    {
                        return Some(*action);
                    }
                    continue;
                }

                return Some(*action);
            }
            None
        })
    }

    /// Visningssträng för en åtgärds genväg (för tooltips)
    pub fn shortcut_hint(&self, action: ShortcutAction) -> Option<String> {
        self.shortcuts.get(&action).map(|s| s.display())
    }

    /// Hämta en kopia av nuvarande genvägar
    pub fn shortcuts(&self) -> &ShortcutMap {
        &self.shortcuts
    }
}

/// Fånga tangentinmatning för genvägsändring i inställningar.
/// Returnerar Some((key, modifiers)) om en tangent trycktes.
pub fn capture_shortcut(ctx: &egui::Context) -> Option<KeyboardShortcut> {
    ctx.input(|input| {
        for event in &input.events {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
            {
                return Some(KeyboardShortcut {
                    key: *key,
                    modifiers: ShortcutModifiers {
                        ctrl_or_cmd: modifiers.command,
                        shift: modifiers.shift,
                        alt: modifiers.alt,
                    },
                });
            }
        }
        None
    })
}
