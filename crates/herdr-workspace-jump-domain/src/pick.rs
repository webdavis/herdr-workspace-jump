use std::fmt::Write;

use crate::JumpTarget;

/// The keystroke that closes the popup without jumping, beside the escape key.
const CANCEL_CHARACTER: char = 'q';

/// One keystroke the pick popup read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKey {
    Character(char),
    Escape,
    /// Ctrl-C, which raw mode delivers as a keystroke rather than a signal.
    Interrupt,
    /// A key that carries no character, such as an arrow or a function key.
    Other,
}

/// What the popup does with the keystroke it read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick<'a> {
    Selected(&'a JumpTarget),
    Cancelled,
    Unbound,
}

/// The popup's menu: one line per workspace, then the cancel line.
pub fn render_menu(targets: &[JumpTarget]) -> String {
    let mut menu = String::new();
    for target in targets {
        let _ = writeln!(menu, "{}  {}", target.pick_key, target.label);
    }
    menu.push_str("esc  cancel\n");
    menu
}

/// Resolve the keystroke against the workspaces the menu offered.
///
/// A workspace that picks on the cancel character is reached, because a key the
/// menu printed beside a label has to do what the menu says.
pub fn decide_pick(targets: &[JumpTarget], key: PickKey) -> Pick<'_> {
    let PickKey::Character(character) = key else {
        return match key {
            PickKey::Escape | PickKey::Interrupt => Pick::Cancelled,
            _ => Pick::Unbound,
        };
    };
    match targets.iter().find(|target| target.pick_key == character) {
        Some(target) => Pick::Selected(target),
        None if character == CANCEL_CHARACTER => Pick::Cancelled,
        None => Pick::Unbound,
    }
}

#[cfg(test)]
mod tests;
