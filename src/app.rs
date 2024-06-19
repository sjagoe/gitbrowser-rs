use git2::Repository;

use ratatui::{
    prelude::Modifier,
    style::{Color, Style},
    text::{Span},
};

pub enum CurrentScreen {
    RefBrowser,
    // TreeBrowser,
    // Pager,
    // Exit,
}

pub struct App {
    pub search_input: String,
    pub current_screen: CurrentScreen,
    pub repo: Repository,
}

impl App {
    pub fn new(repo: Repository) -> App {
        App {
            search_input: String::new(),
            current_screen: CurrentScreen::RefBrowser,
            repo: repo,
        }
    }

    pub fn title(&self) -> Vec<Span> {
        let title = match self.current_screen {
            CurrentScreen::RefBrowser => {
                if let Some(path) = self.repo.path().parent() {
                    if let Some(name) = path.file_name() {
                        format!(" {} ", name.to_string_lossy())
                    } else {
                        format!(" {} ", path.to_string_lossy())
                    }
                } else {
                    format!(" {} ", self.repo.path().to_string_lossy())
                }
            }
        };
        return vec![
            Span::styled(
                title,
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ];
    }

    pub fn items(&self) -> Vec<String> {
        match self.current_screen {
            CurrentScreen::RefBrowser => {
                let mut refs = match self.repo.references() {
                    Ok(r) => r,
                    Err(_e) => return vec![],
                };
                return refs.names().map(|refname| refname.unwrap().to_string()).collect();
            }
        }
    }

    // pub fn save_key_value(&mut self) {
    //     self.pairs
    //         .insert(self.key_input.clone(), self.value_input.clone());

    //     self.key_input = String::new();
    //     self.value_input = String::new();
    //     self.currently_editing = None;
    // }

    // pub fn toggle_editing(&mut self) {
    //     if let Some(edit_mode) = &self.currently_editing {
    //         match edit_mode {
    //             CurrentlyEditing::Key => self.currently_editing = Some(CurrentlyEditing::Value),
    //             CurrentlyEditing::Value => self.currently_editing = Some(CurrentlyEditing::Key),
    //         };
    //     } else {
    //         self.currently_editing = Some(CurrentlyEditing::Key);
    //     }
    // }
}
