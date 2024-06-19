use git2::Repository;

use ratatui::{
    prelude::Modifier,
    style::{Color, Style},
    text::{Span},
};

pub enum CurrentScreen {
    RefBrowser,
    TreeBrowser,
    // Pager,
    // Exit,
}

pub struct App {
    pub selected_index: usize,
    pub search_input: String,
    pub current_screen: CurrentScreen,
    pub repo: Repository,
}

impl App {
    pub fn new(repo: Repository) -> App {
        App {
            selected_index: 0,
            search_input: String::new(),
            current_screen: CurrentScreen::RefBrowser,
            repo: repo,
        }
    }

    pub fn title(&self) -> Vec<Span> {
        let mut parts = vec![
            Span::from(" "),
        ];
        let repo_name = if let Some(path) = self.repo.path().parent() {
            if let Some(name) = path.file_name() {
                format!("{}", name.to_string_lossy())
            } else {
                format!("{}", path.to_string_lossy())
            }
        } else {
            format!("{}", self.repo.path().to_string_lossy())
        };
        parts.push(
            Span::styled(
                repo_name,
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        );
        parts.push(
            Span::from(" "),
        );
        return parts;
    }

    pub fn items(&self) -> Vec<String> {
        match self.current_screen {
            CurrentScreen::RefBrowser => {
                let mut refs = match self.repo.references() {
                    Ok(r) => r,
                    Err(_e) => return vec![],
                };
                return refs.names().map(|refname| refname.unwrap().to_string()).collect();
            },
            _ => vec![],
        }
    }

    pub fn select(&mut self) {
        match self.current_screen {
            CurrentScreen::RefBrowser => {
                self.current_screen = CurrentScreen::TreeBrowser;
                self.selected_index = 0;
            },
            _ => {},
        }
    }

    pub fn back(&mut self) {
        match self.current_screen {
            CurrentScreen::RefBrowser => {
                self.selected_index = 0;
            },
            CurrentScreen::TreeBrowser => {
                self.current_screen = CurrentScreen::RefBrowser;
                self.selected_index = 0;
            },
        }
    }

    pub fn len(&self) -> usize {
        return self.items().len();
    }

    pub fn next_selection(&mut self) {
        if self.selected_index < self.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn previous_selection(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
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
