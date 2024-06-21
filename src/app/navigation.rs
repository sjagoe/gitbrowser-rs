use std::fmt;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub enum NavigationAction {
    Select,
    Back,
    Home,
    End,
    PageUp,
    PageDown,
    NextSelection,
    PreviousSelection,
    ExternalEditor,
    Exit,
    Invalid,
}

pub struct ActionInfo {
    key: String,
    name: String,
}

impl From<KeyEvent> for NavigationAction {
    fn from(key: KeyEvent) -> NavigationAction {
        match (key.code, key.modifiers.bits()) {
            (KeyCode::Enter, 0) => NavigationAction::Select,
            (KeyCode::Home, 0) => NavigationAction::Home,
            (KeyCode::End, 0) => NavigationAction::End,
            (KeyCode::PageUp, 0) => NavigationAction::PageUp,
            (KeyCode::PageDown, 0) => NavigationAction::PageDown,
            (KeyCode::Up, 0) => NavigationAction::PreviousSelection,
            (KeyCode::Down, 0) => NavigationAction::NextSelection,
            (keycode, modifiers) => {
                if modifiers == KeyModifiers::CONTROL.bits() {
                    match keycode {
                        KeyCode::Char('g') => NavigationAction::Back,
                        KeyCode::Char('e') => NavigationAction::ExternalEditor,
                        _ => NavigationAction::Invalid,
                    }
                } else {
                    NavigationAction::Invalid
                }
            }
        }
    }
}

impl From<NavigationAction> for ActionInfo {
    fn from(action: NavigationAction) -> ActionInfo {
        let (key, name) = match action {
            NavigationAction::Select => ("Enter", "Select"),
            NavigationAction::Back => ("C-g", "Back"),
            NavigationAction::Exit => ("C-x", "Exit"),
            NavigationAction::Home => ("Home", "Go to the top"),
            NavigationAction::End => ("End", "Go to the bottom"),
            NavigationAction::PageUp => ("PgUp", "Page Up"),
            NavigationAction::PageDown => ("PgDn", "Page Down"),
            NavigationAction::NextSelection => ("Down", "Select the next item"),
            NavigationAction::PreviousSelection => ("Up", "Select the previous item"),
            NavigationAction::ExternalEditor => ("C-e", "Launch external pager for blob"),
            // We never want to see this but have to define it
            NavigationAction::Invalid => ("invalid", "invalid"),
        };
        ActionInfo {
            key: key.to_string(),
            name: name.to_string(),
        }
    }
}

impl fmt::Display for ActionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.name)
    }
}
