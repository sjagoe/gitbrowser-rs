use git2::{Repository, Object, ObjectType, TreeEntry};

use ratatui::{
    prelude::Modifier,
    style::{Color, Style},
    layout::Rect,
    widgets::Block,
    Frame,
};

pub trait Display {
    fn display_kind<'repo>(&self, repo: &'repo Repository) -> Option<(String, Style)>;

    fn display_name(&self, selected: bool) -> (String, Style);
}

pub trait Navigable<'repo> {
    fn next_selection(&mut self);
    fn previous_selection(&mut self);
    fn select(&self) -> (Object<'repo>, String);
}

pub trait Drawable<'repo> {
    fn draw(&self, f: &mut Frame, area: Rect, content_block: Block, reserved_rows: u16);

    fn title(&self) -> String;
}

impl<'tree> Display for TreeEntry<'tree> {
    fn display_kind<'repo>(&self, repo: &'repo Repository) -> Option<(String, Style)> {
        if let Some(kind) = self.kind() {
            let value = match kind {
                ObjectType::Tree => "tree",
                ObjectType::Blob => {
                    let object = self.to_object(repo).ok()?;
                    let blob = object.peel_to_blob().ok()?;
                    let name = if blob.is_binary() {
                        return Some((
                            "binary".to_string(),
                            Style::default()
                                .fg(Color::Red)
                                .add_modifier(Modifier::DIM),
                        ));
                    } else {
                        "blob"
                    };
                    name
                }
                _ => "unknown",
            };
            return Some((
                value.to_string(),
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::DIM),
            ));
        }
        return Some((
            "unknown".to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
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
