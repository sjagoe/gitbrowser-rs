use git2::{Commit, Object, ObjectType, Repository};

use ratatui::{
    layout::Rect,
    prelude::{Modifier, Text},
    style::{Color, Style},
    text::Span,
    widgets::{
        block::{Padding, Title},
        Block, Borders, Paragraph,
    },
    Frame,
};

use crate::{
    traits::{Drawable, Navigable},
    ui::centered_rect,
};
use color_eyre::Result;

mod blob_pager;
pub mod navigation;
mod pagination;
mod refs_page;
mod tree_page;

use crate::{
    app::{
        blob_pager::BlobPager, navigation::NavigationAction, refs_page::RefsPage,
        tree_page::TreePage,
    },
    errors::GitBrowserError,
};

enum AppMode {
    BrowseRefs,
    BrowseTrees,
    ViewBlob,
    Error,
}

pub struct App<'repo> {
    pub search_input: String,
    repo: &'repo Repository,
    commit: Option<Commit<'repo>>,
    refs_page: RefsPage<'repo>,
    tree_pages: Vec<TreePage<'repo>>,
    blob_pager: Option<BlobPager>,
    mode: Vec<AppMode>,
    height: u16,
    active_error: Option<GitBrowserError>,
}

impl<'repo> App<'repo> {
    pub fn new(repo: &'repo Repository, commit_object: Option<Object<'repo>>) -> App<'repo> {
        let mut new = App {
            search_input: String::new(),
            repo,
            refs_page: RefsPage::new(repo),
            commit: None,
            tree_pages: vec![],
            blob_pager: None,
            mode: vec![AppMode::BrowseRefs],
            height: 0,
            active_error: None,
        };
        if let Some(object) = &commit_object {
            match object.peel_to_commit() {
                Ok(commit) => {
                    new.tree_pages = vec![TreePage::new(repo, object.clone(), "".to_string())];
                    new.mode = vec![AppMode::BrowseTrees];
                    new.commit = Some(commit.clone());
                }
                Err(e) => panic!("Failed to get commit {}", e),
            }
        }

        new
    }

    pub fn set_height(&mut self, h: u16) {
        self.height = h;
    }

    pub fn title(&self) -> Vec<Span> {
        let mut parts = vec![Span::from(" ")];

        parts.push(Span::styled(
            self.refs_page.title(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));

        if let Some(commit) = &self.commit {
            parts.push(Span::styled(
                "@",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
            parts.push(Span::styled(
                commit.id().to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        if self.tree_pages.len() > 1 || self.blob_pager.is_some() {
            parts.push(Span::styled(
                ": ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        }

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

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let title = Title::from(self.title());
        let content_block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .style(Style::default())
            .title(title);

        let viewport = if let Some(page) = match self.mode.last() {
            Some(AppMode::BrowseRefs) => Some(Box::<&dyn Drawable>::new(&self.refs_page)),
            Some(AppMode::BrowseTrees) => Some(Box::<&dyn Drawable>::new(
                self.tree_pages
                    .last()
                    .expect("No tree browsing page in tree mode"),
            )),
            Some(AppMode::ViewBlob) => Some(Box::<&dyn Drawable>::new(
                self.blob_pager
                    .as_ref()
                    .expect("No blob browser page in blob mode"),
            )),
            _ => None,
        } {
            page.draw(f, area, content_block)
        } else {
            content_block.inner(area)
        };

        self.set_height(viewport.height);

        if let Some(error) = self.active_error {
            self.display_error(f, &error);
        }
    }

    fn display_error(&self, f: &mut Frame, error: &GitBrowserError) {
        let area = centered_rect(60, 25, f.size());
        let popup_block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .title(Span::styled(
                " Error ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Color::DarkGray));
        let content = Paragraph::new(Text::styled(
            error.as_str().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .block(popup_block);
        f.render_widget(content, area);
    }

    pub fn navigate(&mut self, action: NavigationAction) -> Result<(), GitBrowserError> {
        // Handle Select and Back on self and exit early
        match action {
            NavigationAction::Select => {
                self.select()?;
                return Ok(());
            }
            NavigationAction::Back => {
                self.back();
                return Ok(());
            }
            _ => {}
        }

        // Handle page navigation
        let page: Box<&mut dyn Navigable> = match self.mode.last() {
            Some(AppMode::BrowseRefs) => Box::new(&mut self.refs_page),
            Some(AppMode::BrowseTrees) => Box::new(
                self.tree_pages
                    .last_mut()
                    .expect("No tree browsing page in tree mode"),
            ),
            Some(AppMode::ViewBlob) => Box::new(
                self.blob_pager
                    .as_mut()
                    .expect("No blob browser page in blob mode"),
            ),
            _ => {
                return Ok(());
            }
        };

        match action {
            NavigationAction::Home => page.home(self.height),
            NavigationAction::End => page.end(self.height),
            NavigationAction::PageUp => page.pageup(self.height),
            NavigationAction::PageDown => page.pagedown(self.height),
            NavigationAction::NextSelection => page.next_selection(),
            NavigationAction::PreviousSelection => page.previous_selection(),
            NavigationAction::Invalid => {}
            // Handled above
            NavigationAction::Select => {}
            NavigationAction::Back => {}
        }
        Ok(())
    }

    pub fn select(&mut self) -> Result<(), GitBrowserError> {
        let page: Box<&mut dyn Navigable> = match self.mode.last() {
            Some(AppMode::BrowseRefs) => Box::new(&mut self.refs_page),
            Some(AppMode::BrowseTrees) => Box::new(
                self.tree_pages
                    .last_mut()
                    .expect("No tree browsing page in tree mode"),
            ),
            Some(AppMode::ViewBlob) => Box::new(
                self.blob_pager
                    .as_mut()
                    .expect("No blob browser page in blob mode"),
            ),
            _ => {
                return Ok(());
            }
        };

        let (object, name) = match page.select() {
            Some(selection) => selection,
            None => return Ok(()),
        };

        match object.kind() {
            Some(ObjectType::Blob) => {
                let pager = BlobPager::from_object(self.repo, object, page.selected_item())?;
                self.blob_pager = Some(pager);
                self.mode.push(AppMode::ViewBlob);
                Ok(())
            }
            Some(ObjectType::Tree) => {
                self.tree_pages.push(TreePage::new(self.repo, object, name));
                self.mode.push(AppMode::BrowseTrees);
                Ok(())
            }
            Some(ObjectType::Commit) => {
                let commit = object.peel_to_commit().expect("Unable to peel commit");
                self.commit = Some(commit);
                self.tree_pages.push(TreePage::new(self.repo, object, name));
                self.mode.push(AppMode::BrowseTrees);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn back(&mut self) {
        if let Some(mode) = self.mode.pop() {
            match mode {
                AppMode::BrowseRefs => {
                    self.tree_pages.pop();
                }
                AppMode::BrowseTrees => {
                    if self.tree_pages.len() > 1 {
                        self.tree_pages.pop();
                    }
                }
                AppMode::ViewBlob => {
                    self.blob_pager = None;
                }
                AppMode::Error => {
                    self.active_error = None;
                }
            }
            if self.mode.is_empty() {
                self.mode.push(mode);
            }
        }
        if self.tree_pages.is_empty() {
            self.commit = None;
        }
    }

    pub fn error(&mut self, error: GitBrowserError) {
        self.active_error = Some(error);
        self.mode.push(AppMode::Error);
    }
}
