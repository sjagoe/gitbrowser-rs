use git2::{Repository, Object};

use ratatui::{
    text::{Line, Span},
    layout::Rect,
    widgets::{
        Block, List, ListItem,
    },
    Frame,
};

use crate::traits::{Display, Drawable, Navigable};
use crate::app::pagination::pagination;

pub struct TreePage<'repo> {
    repo: &'repo Repository,
    tree_object: Object<'repo>,
    selected_index: usize,
    name: String,
}

impl<'repo> TreePage<'repo> {
    pub fn new(repo: &'repo Repository, tree_object: Object<'repo>, name: String) -> TreePage<'repo> {
        TreePage {
            selected_index: 0,
            repo: repo,
            tree_object: tree_object,
            name: name,
        }
    }

    fn len(&self) -> usize {
        match self.tree_object.peel_to_tree() {
            Ok(tree) => {
                return tree.len();
            }
            Err(_) => {
                return 0;
            }
        }
    }
}

impl<'repo> Drawable<'repo> for TreePage<'repo> {
    fn draw(&self, f: &mut Frame, area: Rect, content_block: Block, reserved_rows: u16) {
        match self.tree_object.peel_to_tree() {
            Ok(tree) => {
                let mut list_items = Vec::<ListItem>::new();
                let iter = tree.iter();

                let visible = f.size().height - reserved_rows;
                let (_page, _pages, page_start_index) = pagination(tree.len(), visible.into(), self.selected_index);

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
            }
            Err(e) => {
                panic!("failed to peel tree {}", e);
            }
        }
    }

    fn title(&self) -> String {
        return format!("{}", self.name);
    }
}

impl<'repo> Navigable<'repo> for TreePage<'repo> {
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

    fn select(&self) -> (Object<'repo>, String) {
        match self.tree_object.peel_to_tree() {
            Ok(tree) => {
                if let Some(entry) = tree.get(self.selected_index) {
                    match entry.to_object(self.repo) {
                        Ok(object) => {
                            if let Some(name) = entry.name() {
                                return (object, name.into());
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
                };
            }
            Err(e) => {
                panic!("no tree?!? {}", e);
            }
        }
    }
}
