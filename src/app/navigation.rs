use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};

pub enum NavigationAction {
    Select,
    Back,
    Home,
    End,
    PageUp,
    PageDown,
    NextSelection,
    PreviousSelection,
    Invalid,
}

impl From<KeyEvent> for NavigationAction {
    fn from(key: KeyEvent) -> NavigationAction {
        match (key.code, key.modifiers.bits()) {
            (KeyCode::Enter, 0) => NavigationAction::Select,
            (KeyCode::Char('g'), modifiers) => {
                if modifiers == KeyModifiers::CONTROL.bits() {
                    NavigationAction::Back
                } else {
                    NavigationAction::Invalid
                }
            },
            (KeyCode::Home, 0) => NavigationAction::Home,
            (KeyCode::End, 0) => NavigationAction::End,
            (KeyCode::PageUp, 0) => NavigationAction::PageUp,
            (KeyCode::PageDown, 0) => NavigationAction::PageDown,
            (KeyCode::Up, 0) => NavigationAction::PreviousSelection,
            (KeyCode::Down, 0) => NavigationAction::NextSelection,
            _ => NavigationAction::Invalid,
        }
    }
}
