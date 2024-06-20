use git2::{Object, Repository};
use ratatui::{
    layout::Rect,
    prelude::Modifier,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
    Frame,
};

use crate::app::pagination::pagination;
use crate::traits::{Drawable, Navigable};

pub struct RefsPage<'repo> {
    repo: &'repo Repository,
    selected_index: usize,
}

impl<'repo> RefsPage<'repo> {
    pub fn new(repo: &'repo Repository) -> RefsPage<'repo> {
        RefsPage {
            repo,
            selected_index: 0,
        }
    }

    fn len(&self) -> usize {
        self.repo.references().map(|refs| refs.count()).unwrap_or(0)
    }

    fn items(&self) -> Vec<String> {
        let mut refs = match self.repo.references() {
            Ok(r) => r,
            Err(_e) => return vec![],
        };
        return refs
            .names()
            .map(|refname| refname.expect("Couldn't get refname").to_string())
            .collect();
    }
}

impl<'repo> Drawable<'repo> for RefsPage<'repo> {
    fn draw(&self, f: &mut Frame, area: Rect, content_block: Block, reserved_rows: u16) {
        let mut list_items = Vec::<ListItem>::new();
        let items = self.items();

        let visible = f.size().height - reserved_rows;
        let (_page, _pages, page_start_index) =
            pagination(items.len(), visible.into(), self.selected_index);

        let end_slice = if page_start_index + usize::from(visible) >= items.len() {
            items.len()
        } else {
            page_start_index + usize::from(visible)
        };
        let display_items = &items[page_start_index..end_slice];

        for (pos, item) in display_items.iter().enumerate() {
            let style = if pos + page_start_index == self.selected_index {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };
            let line = Line::from(vec![
                Span::styled(
                    format!("{:10}", "ref"),
                    Style::default().add_modifier(Modifier::DIM),
                ),
                Span::styled(item, style),
            ]);
            list_items.push(ListItem::new(line));
        }
        let content = List::new(list_items).block(content_block);
        f.render_widget(content, area);
    }

    fn title(&self) -> String {
        if let Some(path) = self.repo.path().parent() {
            if let Some(name) = path.file_name() {
                name.to_string_lossy().to_string()
            } else {
                path.to_string_lossy().to_string()
            }
        } else {
            self.repo.path().to_string_lossy().to_string()
        }
    }
}

impl<'repo> Navigable<'repo> for RefsPage<'repo> {
    fn home(&mut self, _page_size: u16) {
        self.selected_index = 0;
    }

    fn end(&mut self, _page_size: u16) {
        self.selected_index = if self.len() > 0 { self.len() - 1 } else { 0 };
    }

    fn pagedown(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        let index = self.selected_index.saturating_add(h);
        self.selected_index = index.min(self.len().saturating_sub(1));
    }

    fn pageup(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        self.selected_index = self.selected_index.saturating_sub(h);
    }

    fn next_selection(&mut self) {
        if self.selected_index < self.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    fn previous_selection(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.len() - 1;
        }
    }

    fn select(&self) -> Option<(Object<'repo>, String)> {
        let selected_ref = &self.items()[self.selected_index];
        match self.repo.revparse_single(selected_ref) {
            Ok(object) => Some((object, "".to_string())),
            Err(e) => {
                panic!("Couldn't parse ref {}", e);
            }
        }
    }

    fn selected_item(&self) -> String {
        let items = &self.items();
        if items.is_empty() {
            "".to_string()
        } else {
            items[self.selected_index].to_string()
        }
    }
}
