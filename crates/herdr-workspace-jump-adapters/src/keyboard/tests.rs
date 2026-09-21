use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::KeyEvent;

use super::*;

fn press(code: KeyCode, modifiers: KeyModifiers) -> Option<PickKey> {
    keystroke_of(Event::Key(KeyEvent::new(code, modifiers)))
}

#[test]
fn a_plain_or_shifted_character_press_is_that_character() {
    assert_eq!(
        press(KeyCode::Char('d'), KeyModifiers::NONE),
        Some(PickKey::Character('d'))
    );
    assert_eq!(
        press(KeyCode::Char('D'), KeyModifiers::SHIFT),
        Some(PickKey::Character('D'))
    );
}

#[test]
fn escape_and_ctrl_c_are_the_two_ways_out() {
    assert_eq!(
        press(KeyCode::Esc, KeyModifiers::NONE),
        Some(PickKey::Escape)
    );
    assert_eq!(
        press(KeyCode::Char('c'), KeyModifiers::CONTROL),
        Some(PickKey::Interrupt)
    );
}

#[test]
fn a_modified_character_or_a_press_carrying_none_has_no_character() {
    for keyless in [
        (KeyCode::Char('d'), KeyModifiers::CONTROL),
        (KeyCode::Char('d'), KeyModifiers::ALT),
        (KeyCode::Up, KeyModifiers::NONE),
        (KeyCode::F(1), KeyModifiers::NONE),
        (KeyCode::Enter, KeyModifiers::NONE),
    ] {
        assert_eq!(
            press(keyless.0, keyless.1),
            Some(PickKey::Other),
            "{keyless:?}"
        );
    }
}

#[test]
fn the_release_of_a_key_is_not_a_keystroke() {
    let release = KeyEvent::new_with_kind(
        KeyCode::Char('d'),
        KeyModifiers::NONE,
        KeyEventKind::Release,
    );

    assert_eq!(keystroke_of(Event::Key(release)), None);
    assert_eq!(
        keystroke_of(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
            KeyEventKind::Repeat,
        ))),
        Some(PickKey::Character('d')),
        "a held key still answers the menu"
    );
}

#[test]
fn a_resize_a_focus_change_and_a_paste_are_waited_through() {
    for ignored in [
        Event::Resize(80, 24),
        Event::FocusGained,
        Event::FocusLost,
        Event::Paste("dotfiles".to_string()),
    ] {
        assert_eq!(keystroke_of(ignored.clone()), None, "{ignored:?}");
    }
}

static RESTORED: AtomicBool = AtomicBool::new(false);

fn mark_restored() {
    RESTORED.store(true, Ordering::Relaxed);
}

#[test]
fn leaving_the_guards_scope_restores_the_terminal() {
    RESTORED.store(false, Ordering::Relaxed);

    drop(RawMode {
        restore: mark_restored,
    });

    assert!(
        RESTORED.load(Ordering::Relaxed),
        "the terminal was left in raw mode"
    );
}
