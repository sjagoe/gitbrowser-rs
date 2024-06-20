use git2::{Repository, ObjectType, TreeEntry};

use ratatui::{
    prelude::Modifier,
    style::{Color, Style},
};

pub trait Display {
    fn display_kind(&self) -> (String, Style);

    fn display_name(&self, selected: bool) -> (String, Style);
}

impl Display for Repository {
    fn display_kind(&self) -> (String, Style) {
        return ("".to_string(), Style::default());
    }

    fn display_name(&self, _selected: bool) -> (String, Style) {
        let repo_name = if let Some(path) = self.path().parent() {
            if let Some(name) = path.file_name() {
                format!("{}", name.to_string_lossy())
            } else {
                format!("{}", path.to_string_lossy())
            }
        } else {
            format!("{}", self.path().to_string_lossy())
        };
        return (
            repo_name,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        );
    }
}

impl<'tree> Display for TreeEntry<'tree> {
    fn display_kind(&self) -> (String, Style) {
        if let Some(kind) = self.kind() {
            let value = match kind {
                ObjectType::Tree => "tree",
                ObjectType::Blob => "blob",
                _ => "unknown",
            };
            return (
                value.to_string(),
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::DIM),
            );
        }
        return (
            "unknown".to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        );
    }

    fn display_name(&self, selected: bool) -> (String, Style) {
        if let Some(kind) = self.kind() {
            let fg = match kind {
                ObjectType::Tree => Color::Blue,
                _ => Color::Gray,
            };
            let mods = match kind {
                ObjectType::Tree => Modifier::BOLD,
                _ => Modifier::empty(),
            };
            let bg = match selected {
                true => Color::Cyan,
                _ => Color::Reset,
            };
            let style = Style::default().fg(fg).bg(bg).add_modifier(mods);
            if let Some(name) = self.name() {
                match kind {
                    ObjectType::Tree => {
                        return (format!("{}/", name), style)
                    }
                    ObjectType::Blob => {
                        return (format!("{}", name), style);
                    }
                    _ => {}
                }
            }
        }
        return ("unknown".into(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
   }
}
