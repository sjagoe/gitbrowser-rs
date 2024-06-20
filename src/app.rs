use git2::{Commit, Object, ObjectType, Repository};

use ratatui::{
    layout::Rect,
    prelude::Modifier,
    style::{Color, Style},
    text::Span,
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

use crate::app::{blob_pager::BlobPager, refs_page::RefsPage, tree_page::TreePage};

enum AppMode {
    ByRef,
    ByCommit,
}

pub struct App<'repo> {
    pub search_input: String,
    repo: &'repo Repository,
    commit: Option<Commit<'repo>>,
    refs_page: RefsPage<'repo>,
    tree_pages: Vec<TreePage<'repo>>,
    blob_pager: Option<BlobPager>,
    mode: AppMode,
    height: u16,
}

impl<'repo> App<'repo> {
    pub fn new(repo: &'repo Repository, commit_object: Option<Object<'repo>>) -> App<'repo> {
        let mut tree_pages: Vec<TreePage<'repo>> = Vec::new();
        if let Some(object) = &commit_object {
            match object.peel_to_commit() {
                Ok(commit) => {
                    tree_pages.push(TreePage::new(repo, object.clone(), "".to_string()));
                    return App {
                        search_input: String::new(),
                        repo: repo,
                        refs_page: RefsPage::new(repo),
                        commit: Some(commit.clone()),
                        tree_pages: tree_pages,
                        blob_pager: None,
                        mode: AppMode::ByCommit,
                        height: 0,
                    };
                }
                Err(e) => panic!("Failed to get commit {}", e),
            }
        }
        return App {
            search_input: String::new(),
            repo: repo,
            refs_page: RefsPage::new(repo),
            commit: None,
            tree_pages: tree_pages,
            blob_pager: None,
            mode: AppMode::ByRef,
            height: 0,
        };
    }

    pub fn set_height(&mut self, h: u16) {
        self.height = h;
    }

    pub fn title(&self) -> Vec<Span> {
        let mut parts = vec![Span::from(" ")];

        let mut repo_name = vec![self.refs_page.title()];
        if let Some(commit) = &self.commit {
            repo_name.push(format!("@{}", commit.id()));
        }
        if self.tree_pages.len() > 1 || self.blob_pager.is_some() {
            repo_name.push(": ".to_string());
        }

        parts.push(Span::styled(
            repo_name.join(""),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));

        for (ix, page) in self.tree_pages.iter().enumerate() {
            let sep = if ix > 0 { "/" } else { "" };
            parts.push(Span::styled(
                format!("{}{}", page.title(), sep),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        if let Some(pager) = &self.blob_pager {
            parts.push(Span::styled(
                pager.title(),
                Style::default().fg(Color::Gray),
            ));
        }

        parts.push(Span::from(" "));
        parts
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

    pub fn home(&mut self) {
        let page: Box<&mut dyn Navigable> = if let Some(pager) = &mut self.blob_pager {
            Box::new(pager)
        } else if let Some(p) = self.tree_pages.last_mut() {
            Box::new(p)
        } else {
            Box::new(&mut self.refs_page)
        };
        page.home(self.height);
    }

    pub fn end(&mut self) {
        let page: Box<&mut dyn Navigable> = if let Some(pager) = &mut self.blob_pager {
            Box::new(pager)
        } else if let Some(p) = self.tree_pages.last_mut() {
            Box::new(p)
        } else {
            Box::new(&mut self.refs_page)
        };
        page.end(self.height);
    }

    pub fn pagedown(&mut self) {
        let page: Box<&mut dyn Navigable> = if let Some(pager) = &mut self.blob_pager {
            Box::new(pager)
        } else if let Some(p) = self.tree_pages.last_mut() {
            Box::new(p)
        } else {
            Box::new(&mut self.refs_page)
        };
        page.pagedown(self.height);
    }

    pub fn pageup(&mut self) {
        let page: Box<&mut dyn Navigable> = if let Some(pager) = &mut self.blob_pager {
            Box::new(pager)
        } else if let Some(p) = self.tree_pages.last_mut() {
            Box::new(p)
        } else {
            Box::new(&mut self.refs_page)
        };
        page.pageup(self.height);
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
                        if let Some(pager) =
                            BlobPager::from_object(self.repo, object, page.selected_item())
                        {
                            self.blob_pager = Some(pager);
                        }
                    }
                    Some(ObjectType::Tree) => {
                        self.tree_pages.push(TreePage::new(self.repo, object, name));
                    }
                    Some(ObjectType::Commit) => match object.peel_to_commit() {
                        Ok(commit) => {
                            self.commit = Some(commit);
                            self.tree_pages.push(TreePage::new(self.repo, object, name));
                        }
                        Err(e) => panic!("Unable to peel commit? {}", e),
                    },
                    _ => {}
                }
            }
        }
    }

    pub fn back(&mut self) {
        if self.blob_pager.is_none() {
            match self.mode {
                AppMode::ByRef => {
                    self.tree_pages.pop();
                }
                AppMode::ByCommit => {
                    if self.tree_pages.len() > 1 {
                        self.tree_pages.pop();
                    }
                }
            }
            if self.tree_pages.is_empty() {
                self.commit = None;
            }
        } else {
            self.blob_pager = None;
        }
    }
}
