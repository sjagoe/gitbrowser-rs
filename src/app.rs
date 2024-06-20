use git2::{Repository, Object, Tree};

use ratatui::{
    prelude::Modifier,
    style::{Color, Style},
    text::{Line, Span},
    layout::Rect,
    widgets::{
        block::{Padding, Title},
        Block, Borders, List, ListItem, Paragraph, Widget,
    },
    Frame,
};

use crate::display::Display;

struct RefsPage<'repo> {
    repo: &'repo Repository,
    selected_index: usize,
}

#[derive(Clone)]
struct TreePage<'repo> {
    repo: &'repo Repository,
    tree_object: Object<'repo>,
    selected_index: usize,
    name: String,
}

pub struct App<'repo> {
    pub search_input: String,
    refs_page: RefsPage<'repo>,
    tree_pages: Vec<TreePage<'repo>>,
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

    pub fn draw(&self, f: &mut Frame, area: Rect, content_block: Block, reserved_rows: u16) {
        match self.tree_object.peel_to_tree() {
            Ok(tree) => {
                let mut list_items = Vec::<ListItem>::new();
                let iter = tree.iter();

                let visible = f.size().height - reserved_rows;
                let (_page, _pages, page_start_index) = pagination(tree.len(), visible.into(), self.selected_index);

                let display_items = iter.skip(page_start_index).take(visible.into());

                for (pos, entry) in display_items.enumerate() {
                    let selected = pos + page_start_index == self.selected_index;
                    let (value, style) = entry.display_name(selected);
                    let line = Line::from(Span::styled(value, style));
                    list_items.push(ListItem::new(line));
                }
                let content = List::new(list_items).block(content_block);
                f.render_widget(content, area);
            }
            Err(e) => {
                panic!("failed to peel tree {}", e);
            }
        }
    }

    pub fn title(&self) -> String {
        return format!("{}", self.name);
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

    pub fn select(&mut self) -> TreePage<'repo> {
        match self.tree_object.peel_to_tree() {
            Ok(tree) => {
                if let Some(entry) = tree.get(self.selected_index) {
                    match entry.to_object(self.repo) {
                        Ok(object) => {
                            if let Some(name) = entry.name() {
                                let page = TreePage::new(
                                    self.repo,
                                    object,
                                    name.into(),
                                );
                                return page;
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

impl<'repo> RefsPage<'repo> {
    pub fn new(repo: &'repo Repository) -> RefsPage<'repo> {
        RefsPage {
            repo: repo,
            selected_index: 0,
        }
    }

    fn items(&self) -> Vec<String> {
        let mut refs = match self.repo.references() {
            Ok(r) => r,
            Err(_e) => return vec![],
        };
        return refs.names().map(|refname| refname.unwrap().to_string()).collect();
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, content_block: Block, reserved_rows: u16) {
        let mut list_items = Vec::<ListItem>::new();
        let items = self.items();

        let visible = f.size().height - reserved_rows;
        let (_page, _pages, page_start_index) = pagination(items.len(), visible.into(), self.selected_index);

        let end_slice = if page_start_index + usize::from(visible) >= items.len() {
            items.len()
        } else {
            page_start_index + usize::from(visible)
        };
        let display_items = &items[page_start_index .. end_slice];

        for (pos, item) in display_items.iter().enumerate() {
            let style = if pos + page_start_index == self.selected_index {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };
            list_items.push(ListItem::new(Line::from(Span::styled(item, style))));
        }
        let content = List::new(list_items).block(content_block);
        f.render_widget(content, area);
    }

    pub fn title(&self) -> String {
        if let Some(path) = self.repo.path().parent() {
            if let Some(name) = path.file_name() {
                return format!("{}", name.to_string_lossy());
            } else {
                return format!("{}", path.to_string_lossy());
            }
        } else {
            return format!("{}", self.repo.path().to_string_lossy());
        };
    }

    pub fn next_selection(&mut self) {
        if self.selected_index < self.items().len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn previous_selection(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn select(&mut self) -> TreePage<'repo> {
        let selected_ref = &self.items()[self.selected_index];
        match self.repo.revparse_single(selected_ref) {
            Ok(object) => {
                let page = TreePage::new(
                    self.repo,
                    object,
                    "".into(),
                );
                return page;
            }
            Err(e) => {
                panic!("Couldn't parse ref {}", e);
            }
        }

    }

    pub fn back(&mut self) {}
}


impl<'repo> App<'repo> {
    pub fn new(repo: &'repo Repository) -> App<'repo> {
        App {
            search_input: String::new(),
            refs_page: RefsPage::new(repo),
            tree_pages: vec![],
        }
    }

    pub fn title(&self) -> Vec<Span> {
        let mut parts = vec![
            Span::from(" "),
        ];
        parts.push(
            Span::styled(
                self.refs_page.title(),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        );
        parts.push(Span::from(" "));
        for page in self.tree_pages.iter() {
            parts.push(
                Span::styled(
                    format!("{}/", page.title()),
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
            );
        }
        parts.push(Span::from(" "));
        return parts;
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, reserved_rows: u16) {
        let title = Title::from(self.title());
        let content_block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .style(Style::default())
            .title(title);

        if let Some(page) = self.tree_pages.last() {
            page.draw(f, area, content_block, reserved_rows);
        } else {
            self.refs_page.draw(f, area, content_block, reserved_rows);
        }
    }

    pub fn next_selection(&mut self) {
        if let Some(page) = self.tree_pages.last_mut() {
            page.next_selection();
        } else {
            self.refs_page.next_selection();
        }
    }

    pub fn previous_selection(&mut self) {
        if let Some(page) = self.tree_pages.last_mut() {
            page.previous_selection();
        } else {
            self.refs_page.previous_selection();
        }
    }

    pub fn select(&mut self) {
        if let Some(page) = self.tree_pages.last_mut() {
            let new_page = page.select();
            self.tree_pages.push(new_page);
        } else {
            let new_page = self.refs_page.select();
            self.tree_pages.push(new_page);
        }
    }

    pub fn back(&mut self) {
        if self.tree_pages.pop().is_none() {
            self.refs_page.back();
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

fn pagination(item_count: usize, visible_item_count: usize, selected_index: usize) -> (usize, usize, usize) {
    let page_start_index = selected_index - (selected_index % visible_item_count);
    let pages = if item_count % visible_item_count > 0 {
        item_count / visible_item_count + 1
    } else {
        item_count / visible_item_count
    };
    let page = if page_start_index % visible_item_count > 0 {
        page_start_index / visible_item_count + 1
    } else {
        page_start_index / visible_item_count
    };
    (page, pages, page_start_index)
}
