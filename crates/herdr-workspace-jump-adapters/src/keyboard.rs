//! The one keystroke the pick popup reads, in raw mode.

use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use herdr_workspace_jump_domain::PickKey;

/// Raw mode for as long as this value lives.
///
/// The popup borrows the terminal herdr gave it, so raw mode is restored on the
/// way out of every branch, including a failed read.
struct RawMode;

impl RawMode {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

/// Read one keystroke from the terminal, ignoring everything that is not a key press.
pub fn read_one_key() -> io::Result<PickKey> {
    let _raw_mode = RawMode::enter()?;
    loop {
        if let Event::Key(key) = read()?
            && key.kind != KeyEventKind::Release
        {
            return Ok(pick_key(key));
        }
    }
}

fn pick_key(key: KeyEvent) -> PickKey {
    let control_or_alt = KeyModifiers::CONTROL | KeyModifiers::ALT;
    match key.code {
        KeyCode::Esc => PickKey::Escape,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => PickKey::Interrupt,
        KeyCode::Char(character) if !key.modifiers.intersects(control_or_alt) => {
            PickKey::Character(character)
        }
        _ => PickKey::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_or_shifted_character_press_is_that_character() {
        assert_eq!(
            pick_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)),
            PickKey::Character('d')
        );
        assert_eq!(
            pick_key(KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT)),
            PickKey::Character('D')
        );
    }

    #[test]
    fn escape_and_ctrl_c_are_the_two_ways_out() {
        assert_eq!(
            pick_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            PickKey::Escape
        );
        assert_eq!(
            pick_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            PickKey::Interrupt
        );
    }

    #[test]
    fn a_modified_character_or_a_press_carrying_none_has_no_character() {
        for keyless in [
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ] {
            assert_eq!(pick_key(keyless), PickKey::Other, "{keyless:?}");
        }
    }
}
