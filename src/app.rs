use git2::{Commit, Repository, ObjectType};

use ratatui::{
    prelude::Modifier,
    style::{Color, Style},
    text::{Span},
    layout::Rect,
    widgets::{
        block::{Padding, Title},
        Block, Borders,
    },
    Frame,
};

use crate::traits::{Drawable, Navigable};

mod blob_pager;
mod pagination;
mod refs_page;
mod tree_page;

use crate::app::{
    blob_pager::BlobPager,
    refs_page::RefsPage,
    tree_page::TreePage,
};

pub struct App<'repo> {
    pub search_input: String,
    repo: &'repo Repository,
    commit: Option<Commit<'repo>>,
    refs_page: RefsPage<'repo>,
    tree_pages: Vec<TreePage<'repo>>,
    blob_pager: Option<BlobPager<'repo>>,
}

impl<'repo> App<'repo> {
    pub fn new(repo: &'repo Repository) -> App<'repo> {
        App {
            search_input: String::new(),
            repo: repo,
            refs_page: RefsPage::new(repo),
            commit: None,
            tree_pages: vec![],
            blob_pager: None,
        }
    }

    pub fn set_height(&mut self, h: u16) {
        if let Some(page) = &mut self.blob_pager {
            page.set_height(h);
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
        if self.tree_pages.len() > 1 || !self.blob_pager.is_none() {
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

        if let Some(pager) = &self.blob_pager {
            parts.push(
                Span::styled(
                    pager.title(),
                    Style::default().fg(Color::Gray),
                )
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

        let page: Box<&dyn Drawable> = if let Some(p) = &self.blob_pager {
            Box::new(p)
        } else if let Some(p) = self.tree_pages.last() {
            Box::new(p)
        } else {
            Box::new(&self.refs_page)
        };

        page.draw(f, area, content_block, reserved_rows);
    }

    pub fn pagedown(&mut self) {
        let page: Box<&mut dyn Navigable> = if let Some(pager) = &mut self.blob_pager {
            Box::new(pager)
        } else if let Some(p) = self.tree_pages.last_mut() {
            Box::new(p)
        } else {
            Box::new(&mut self.refs_page)
        };
        page.pagedown();
    }

    pub fn pageup(&mut self) {
        let page: Box<&mut dyn Navigable> = if let Some(pager) = &mut self.blob_pager {
            Box::new(pager)
        } else if let Some(p) = self.tree_pages.last_mut() {
            Box::new(p)
        } else {
            Box::new(&mut self.refs_page)
        };
        page.pageup();
    }

    pub fn next_selection(&mut self) {
        let page: Box<&mut dyn Navigable> = if let Some(pager) = &mut self.blob_pager {
            Box::new(pager)
        } else if let Some(p) = self.tree_pages.last_mut() {
            Box::new(p)
        } else {
            Box::new(&mut self.refs_page)
        };
        page.next_selection();
    }

    pub fn previous_selection(&mut self) {
        let page: Box<&mut dyn Navigable> = if let Some(pager) = &mut self.blob_pager {
            Box::new(pager)
        } else if let Some(p) = self.tree_pages.last_mut() {
            Box::new(p)
        } else {
            Box::new(&mut self.refs_page)
        };
        page.previous_selection();
    }

    pub fn select(&mut self) {
        if self.blob_pager.is_none() {
            let page: Box<&dyn Navigable> = if let Some(p) = self.tree_pages.last() {
                Box::new(p)
            } else {
                Box::new(&self.refs_page)
            };
            if let Some((object, name)) = page.select() {
                match object.kind() {
                    Some(ObjectType::Blob) => {
                        self.blob_pager = Some(BlobPager::from_object(self.repo, object, page.selected_item()));
                    },
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
        }
    }

    pub fn back(&mut self) {
        if self.blob_pager.is_none() {
            self.tree_pages.pop();
            if self.tree_pages.len() == 0 {
                self.commit = None;
            }
        } else {
            self.blob_pager = None;
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
