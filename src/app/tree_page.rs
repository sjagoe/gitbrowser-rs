use git2::{Object, Repository};

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, List, ListItem},
    Frame,
};

use crate::app::pagination::pagination;
use crate::traits::{Display, Drawable, Navigable};

pub struct TreePage<'repo> {
    repo: &'repo Repository,
    tree_object: Object<'repo>,
    selected_index: usize,
    name: String,
}

impl<'repo> TreePage<'repo> {
    pub fn new(
        repo: &'repo Repository,
        tree_object: Object<'repo>,
        name: String,
    ) -> TreePage<'repo> {
        TreePage {
            selected_index: 0,
            repo,
            tree_object,
            name,
        }
    }

    fn len(&self) -> usize {
        match self.tree_object.peel_to_tree() {
            Ok(tree) => tree.len(),
            Err(_) => 0,
        }
    }
}

impl<'repo> Drawable<'repo> for TreePage<'repo> {
    fn draw(&self, f: &mut Frame, area: Rect, content_block: Block) -> Rect {
        match self.tree_object.peel_to_tree() {
            Ok(tree) => {
                let viewport = content_block.inner(area);
                let mut list_items = Vec::<ListItem>::new();
                let iter = tree.iter();

                let visible = viewport.height;
                let (_page, _pages, page_start_index) =
                    pagination(tree.len(), visible.into(), self.selected_index);

                let display_items = iter.skip(page_start_index).take(visible.into());

                for (pos, entry) in display_items.enumerate() {
                    let selected = pos + page_start_index == self.selected_index;
                    if let Some((kind, kind_style)) = entry.display_kind(self.repo) {
                        let (value, style) = entry.display_name(selected);
                        let line = Line::from(vec![
                            Span::styled(format!("{:10}", kind), kind_style),
                            Span::styled(value, style),
                        ]);
                        list_items.push(ListItem::new(line));
                    }
                }
                let content = List::new(list_items).block(content_block);
                f.render_widget(content, area);
                viewport
            }
            Err(e) => {
                panic!("failed to peel tree {}", e);
            }
        }
    }

    fn title(&self) -> String {
        self.name.to_string()
    }
}

impl<'repo> Navigable<'repo> for TreePage<'repo> {
    fn home(&mut self, _page_size: u16) {
        self.selected_index = 0;
    }

    fn end(&mut self, _page_size: u16) {
        self.selected_index = if self.len() > 0 { self.len() - 1 } else { 0 };
    }

    fn pagedown(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        let selected_index = self.selected_index + h;
        self.selected_index = if selected_index > self.len() {
            self.len() - 1
        } else {
            selected_index
        }
    }

    fn pageup(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        if self.selected_index < h {
            self.selected_index = 0;
        } else {
            self.selected_index -= h;
        }
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
        match self.tree_object.peel_to_tree() {
            Ok(tree) => {
                if let Some(entry) = tree.get(self.selected_index) {
                    match entry.to_object(self.repo) {
                        Ok(object) => {
                            if let Some(name) = entry.name() {
                                Some((object, name.into()))
                            } else {
                                panic!("Failed to get tree entry name");
                            }
                        }
                        Err(e) => {
                            panic!("Failed to get object from entry {}", e);
                        }
                    }
                } else {
                    panic!("no tree entry?!?");
                }
            }
            Err(e) => {
                panic!("no tree?!? {}", e);
            }
        }
    }

    fn selected_item(&self) -> String {
        match self.tree_object.peel_to_tree() {
            Ok(tree) => {
                if let Some(entry) = tree.get(self.selected_index) {
                    if let Some(name) = entry.name() {
                        name.to_string()
                    } else {
                        panic!("Failed to get tree entry name");
                    }
                } else {
                    panic!("no tree entry?!?");
                }
            }
            Err(e) => {
                panic!("no tree?!? {}", e);
            }
        }
    }
}
