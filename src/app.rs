use git2::{Commit, Repository, Object, ObjectType};

use ratatui::{
    prelude::Modifier,
    style::{Color, Style},
    text::{Line, Span},
    layout::Rect,
    widgets::{
        block::{Padding, Title},
        Block, Borders, List, ListItem,
    },
    Frame,
};

use crate::traits::{Drawable, Navigable};

mod tree_page;
mod pagination;

use crate::app::tree_page::TreePage;
use crate::app::pagination::pagination;

struct RefsPage<'repo> {
    repo: &'repo Repository,
    selected_index: usize,
}

pub struct App<'repo> {
    pub search_input: String,
    repo: &'repo Repository,
    commit: Option<Commit<'repo>>,
    refs_page: RefsPage<'repo>,
    tree_pages: Vec<TreePage<'repo>>,
}

impl<'repo> RefsPage<'repo> {
    pub fn new(repo: &'repo Repository) -> RefsPage<'repo> {
        RefsPage {
            repo: repo,
            selected_index: 0,
        }
    }

    fn len(&self) -> usize {
        match self.repo.references() {
            Ok(refs) => refs.count(),
            Err(_) => 0,
        }
    }

    fn items(&self) -> Vec<String> {
        let mut refs = match self.repo.references() {
            Ok(r) => r,
            Err(_e) => return vec![],
        };
        return refs.names().map(|refname| refname.unwrap().to_string()).collect();
    }
}

impl<'repo> Drawable<'repo> for RefsPage<'repo> {
    fn draw(&self, f: &mut Frame, area: Rect, content_block: Block, reserved_rows: u16) {
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
            let line = Line::from(vec![
                Span::styled(format!("{:10}", "ref"), Style::default().add_modifier(Modifier::DIM)),
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
                return format!("{}", name.to_string_lossy());
            } else {
                return format!("{}", path.to_string_lossy());
            }
        } else {
            return format!("{}", self.repo.path().to_string_lossy());
        };
    }
}

impl<'repo> Navigable<'repo> for RefsPage<'repo> {
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
        let selected_ref = &self.items()[self.selected_index];
        match self.repo.revparse_single(selected_ref) {
            Ok(object) => {
                return (object, "".into());
            }
            Err(e) => {
                panic!("Couldn't parse ref {}", e);
            }
        }

    }
}


impl<'repo> App<'repo> {
    pub fn new(repo: &'repo Repository) -> App<'repo> {
        App {
            search_input: String::new(),
            repo: repo,
            refs_page: RefsPage::new(repo),
            commit: None,
            tree_pages: vec![],
        }
    }

    pub fn title(&self) -> Vec<Span> {
        let mut parts = vec![
            Span::from(" "),
        ];

        let mut repo_name = vec![self.refs_page.title()];
        if let Some(commit) = &self.commit {
            repo_name.push(format!("@{}", commit.id()));
        }
        if self.tree_pages.len() > 1 {
            repo_name.push(": ".to_string());
        }

        parts.push(
            Span::styled(
                repo_name.join(""),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        );

        for (ix, page) in self.tree_pages.iter().enumerate() {
            let sep = if ix > 0 { "/" } else { "" };
            parts.push(
                Span::styled(
                    format!("{}{}", page.title(), sep),
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

        let page: Box<&dyn Drawable> = if let Some(p) = self.tree_pages.last() {
            Box::new(p)
        } else {
            Box::new(&self.refs_page)
        };

        page.draw(f, area, content_block, reserved_rows);
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
        let page: Box<&dyn Navigable> = if let Some(p) = self.tree_pages.last() {
            Box::new(p)
        } else {
            Box::new(&self.refs_page)
        };
        let (object, name) = page.select();
        match object.kind() {
            Some(ObjectType::Blob) => {},
            Some(ObjectType::Tree) => {
                self.tree_pages.push(
                    TreePage::new(
                        self.repo,
                        object,
                        name,
                    ),
                );
            }
            Some(ObjectType::Commit) => {
                match object.peel_to_commit() {
                    Ok(commit) => {
                        self.commit = Some(commit);
                        self.tree_pages.push(
                            TreePage::new(
                                self.repo,
                                object,
                                name,
                            ),
                        );
                    }
                    Err(e) => panic!("Unable to peel commit? {}", e)
                }
            }
            _ => {}
        }
    }

    pub fn back(&mut self) {
        self.tree_pages.pop();
        if self.tree_pages.len() == 0 {
            self.commit = None;
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
