use git2::{Commit, Object, ObjectType, Repository};

use ratatui::{
    layout::Rect,
    prelude::{Modifier, Text},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        block::{Padding, Title},
        Block, Borders, Paragraph,
    },
    Frame,
};

use color_eyre::Result;

use syntect::highlighting;
use syntect::parsing::SyntaxSet;

use crate::{
    traits::{Drawable, Navigable},
    ui::centered_rect,
};

mod blob_pager;
mod external_editor;
pub mod navigation;
mod pagination;
mod refs_page;
mod tree_page;

use crate::{
    app::{
        blob_pager::BlobPager,
        external_editor::ExternalEditor,
        navigation::{ActionInfo, NavigationAction},
        refs_page::RefsPage,
        tree_page::TreePage,
    },
    errors::{ErrorKind, GitBrowserError},
};

#[derive(Debug)]
pub enum AppMode {
    BrowseRefs,
    BrowseTrees,
    ViewBlob,
    ExternalEditor,
    Error,
}

pub struct App<'repo, 'syntax> {
    pub search_input: String,
    repo: &'repo Repository,
    commit: Option<Commit<'repo>>,
    refs_page: RefsPage<'repo>,
    tree_pages: Vec<TreePage<'repo>>,
    blob_pager: Option<BlobPager<'repo, 'syntax>>,
    external_editor: Option<ExternalEditor>,
    mode_history: Vec<AppMode>,
    height: u16,
    active_error: Option<GitBrowserError>,
    editor: String,
    syntax_set: &'syntax SyntaxSet,
    theme: &'syntax highlighting::Theme,
}

pub struct Redraw(pub bool);

impl<'repo, 'syntax> App<'repo, 'syntax> {
    pub fn new(
        repo: &'repo Repository,
        commit_object: Option<Object<'repo>>,
        editor: String,
        syntax_set: &'syntax SyntaxSet,
        theme: &'syntax highlighting::Theme,
    ) -> App<'repo, 'syntax> {
        let mut new = App {
            search_input: String::new(),
            repo,
            commit: None,
            refs_page: RefsPage::new(repo),
            tree_pages: vec![],
            blob_pager: None,
            external_editor: None,
            mode_history: vec![AppMode::BrowseRefs],
            height: 0,
            active_error: None,
            editor,
            syntax_set,
            theme,
        };
        if let Some(object) = &commit_object {
            match object.peel_to_commit() {
                Ok(commit) => {
                    new.tree_pages = vec![TreePage::new(repo, object.clone(), "".to_string())];
                    new.mode_history = vec![AppMode::BrowseTrees];
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

    pub fn draw_context_hint(&self, f: &mut Frame, area: Rect) {
        let actions = match self.mode() {
            AppMode::BrowseRefs => {
                vec![
                    NavigationAction::Exit,
                    NavigationAction::Back,
                    NavigationAction::Select,
                ]
            }
            AppMode::ViewBlob => {
                vec![
                    NavigationAction::Exit,
                    NavigationAction::Back,
                    NavigationAction::ExternalEditor,
                ]
            }
            _ => {
                vec![
                    NavigationAction::Exit,
                    NavigationAction::Back,
                    NavigationAction::Select,
                    NavigationAction::ExternalEditor,
                ]
            }
        };
        let keys_hint = actions
            .iter()
            .map(|a| ActionInfo::from(a).to_string())
            .collect::<Vec<String>>()
            .join(" | ");
        let content = Span::styled(keys_hint, Style::default());
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .style(Style::default().fg(Color::Black).bg(Color::Gray));
        let hint = Paragraph::new(Line::from(content)).block(block);
        f.render_widget(hint, area);
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let title = Title::from(self.title());
        let content_block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .style(Style::default())
            .title(title);

        let viewport = if let Some(page) = match self.mode() {
            AppMode::BrowseRefs => Some(Box::<&dyn Drawable>::new(&self.refs_page)),
            AppMode::BrowseTrees => Some(Box::<&dyn Drawable>::new(
                self.tree_pages
                    .last()
                    .expect("No tree browsing page in tree mode"),
            )),
            AppMode::ViewBlob => Some(Box::<&dyn Drawable>::new(
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

    pub fn navigate(&mut self, action: &NavigationAction) -> Result<Redraw, GitBrowserError> {
        // Handle Select and Back on self and exit early
        match (action, self.mode()) {
            (NavigationAction::ExternalEditor, AppMode::BrowseTrees) => {
                return self.view_blob();
            }
            (NavigationAction::Select, _) => {
                self.select()?;
                return Ok(Redraw(false));
            }
            (NavigationAction::Back, _) => {
                self.back();
                return Ok(Redraw(false));
            }
            _ => {}
        }

        // Handle page navigation
        let page: Box<&mut dyn Navigable> = match self.mode() {
            AppMode::BrowseRefs => Box::new(&mut self.refs_page),
            AppMode::BrowseTrees => Box::new(
                self.tree_pages
                    .last_mut()
                    .expect("No tree browsing page in tree mode"),
            ),
            AppMode::ViewBlob => Box::new(
                self.blob_pager
                    .as_mut()
                    .expect("No blob browser page in blob mode"),
            ),
            _ => {
                return Ok(Redraw(false));
            }
        };

        match action {
            NavigationAction::Tick => {
                page.next_tick(false)?;
            }
            NavigationAction::Home => page.home(self.height),
            NavigationAction::End => {
                page.next_tick(true)?;
                page.end(self.height);
            }
            NavigationAction::PageUp => page.pageup(self.height),
            NavigationAction::PageDown => page.pagedown(self.height),
            NavigationAction::NextSelection => page.next_selection(),
            NavigationAction::PreviousSelection => page.previous_selection(),
            NavigationAction::ExternalEditor => {
                return self.view_blob();
            }
            NavigationAction::Invalid => {}
            // Handled above
            NavigationAction::Select => {}
            NavigationAction::Back => {}
            // Handled outside of app
            NavigationAction::Exit => {}
        }
        Ok(Redraw(false))
    }

    pub fn select(&mut self) -> Result<(), GitBrowserError> {
        let page: Box<&mut dyn Navigable> = match self.mode() {
            AppMode::BrowseRefs => Box::new(&mut self.refs_page),
            AppMode::BrowseTrees => Box::new(
                self.tree_pages
                    .last_mut()
                    .expect("No tree browsing page in tree mode"),
            ),
            AppMode::ViewBlob => Box::new(
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
                let pager = BlobPager::from_object(
                    self.repo,
                    object,
                    page.selected_item(),
                    self.syntax_set,
                    self.theme,
                )?;
                self.blob_pager = Some(pager);
                self.mode_history.push(AppMode::ViewBlob);
                Ok(())
            }
            Some(ObjectType::Tree) => {
                self.tree_pages.push(TreePage::new(self.repo, object, name));
                self.mode_history.push(AppMode::BrowseTrees);
                Ok(())
            }
            Some(ObjectType::Commit) => {
                let commit = object.peel_to_commit().expect("Unable to peel commit");
                self.commit = Some(commit);
                self.tree_pages.push(TreePage::new(self.repo, object, name));
                self.mode_history.push(AppMode::BrowseTrees);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn back(&mut self) {
        if let Some(mode) = self.mode_history.pop() {
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
                AppMode::ExternalEditor => {
                    self.external_editor = None;
                }
                AppMode::Error => {
                    self.active_error = None;
                }
            }
            if self.mode_history.is_empty() {
                self.mode_history.push(mode);
            }
        }
        if self.tree_pages.is_empty() {
            self.commit = None;
        }
    }

    pub fn error(&mut self, error: GitBrowserError) {
        self.active_error = Some(error);
        self.mode_history.push(AppMode::Error);
    }

    pub fn view_blob(&mut self) -> Result<Redraw, GitBrowserError> {
        self.external_editor = match self.mode() {
            AppMode::ViewBlob => {
                if let Some(pager) = &self.blob_pager {
                    Some(ExternalEditor::new(&pager.blob, &pager.name, &self.editor))
                } else {
                    return Ok(Redraw(false));
                }
            }
            AppMode::BrowseTrees => {
                let page = match self.tree_pages.last() {
                    Some(page) => page,
                    None => return Ok(Redraw(false)),
                };

                let (object, name) = match page.select() {
                    Some(selection) => selection,
                    None => return Ok(Redraw(false)),
                };

                if !matches!(object.kind(), Some(ObjectType::Blob)) {
                    return Ok(Redraw(false));
                }

                let blob = object
                    .into_blob()
                    .map_err(|_| GitBrowserError::Error(ErrorKind::BlobReference))?;

                Some(ExternalEditor::new(&blob, &name, &self.editor))
            }
            _ => return Ok(Redraw(false)),
        };
        self.mode_history.push(AppMode::ExternalEditor);
        if let Some(external_editor) = &mut self.external_editor {
            if let Some(e) = external_editor.display().err() {
                self.back();
                return Err(e);
            };
        }
        // We need to go back to the previous mode after the blocking editor display
        self.back();
        Ok(Redraw(true))
    }

    pub fn mode(&self) -> &AppMode {
        self.mode_history.last().expect("no application mode found")
    }
}
