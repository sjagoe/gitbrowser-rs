use git2::{Blob, Object, Repository};

use ratatui::{
    layout::Rect,
    prelude::{Line, Modifier, Span, Style},
    widgets::{Block, Paragraph},
    Frame,
};

use color_eyre::Result;

use crate::errors::{ErrorKind, GitBrowserError};
use crate::traits::{Drawable, Navigable};

pub struct BlobPager<'repo> {
    top: usize,
    // repo: &'repo Repository,
    pub blob: Blob<'repo>,
    pub name: String,
    lines: Vec<String>,
}

impl<'repo> BlobPager<'repo> {
    pub fn new(_repo: &'repo Repository, blob: Blob<'repo>, name: String) -> BlobPager<'repo> {
        let content = match std::str::from_utf8(blob.content()) {
            Ok(v) => v,
            Err(e) => panic!("unable to decode utf8 {}", e),
        };
        let lines = content.lines().map(|line| line.to_string()).collect();
        BlobPager {
            top: 0,
            // repo: repo,
            blob: blob.clone(),
            name,
            lines,
        }
    }

    pub fn from_object(
        repo: &'repo Repository,
        object: Object<'repo>,
        name: String,
    ) -> Result<Self, GitBrowserError> {
        match object.into_blob() {
            Ok(blob) => {
                if blob.is_binary() {
                    Err(GitBrowserError::Error(ErrorKind::BinaryFile))
                } else {
                    Ok(BlobPager::new(repo, blob, name))
                }
            }
            Err(_) => Err(GitBrowserError::Error(ErrorKind::BlobReference)),
        }
    }
}

impl<'repo> Drawable<'repo> for BlobPager<'repo> {
    fn draw(&self, f: &mut Frame, area: Rect, content_block: Block) -> Rect {
        let viewport = content_block.inner(area);
        let height: usize = viewport.height.into();
        let bottom = if self.top + height > self.lines.len() {
            self.lines.len()
        } else {
            self.top + height
        };
        let filler: Vec<Line> = if bottom - self.top < height {
            let v: Vec<Line> = vec![Line::styled(
                "~",
                Style::default().add_modifier(Modifier::DIM),
            )];
            let len = height - (bottom - self.top);
            v.iter().cycle().take(len).cloned().collect()
        } else {
            vec![]
        };
        let lines: Vec<Line> = self.lines[self.top..bottom]
            .iter()
            .enumerate()
            .map(|(index, text)| {
                let tmp = format!("{}", bottom);
                let width = tmp.len();
                let formatted = format!("{:width$} | ", index + self.top);
                let lineno = Span::styled(formatted, Style::default().add_modifier(Modifier::DIM));
                Line::from(vec![lineno, Span::from(text)])
            })
            .collect();
        let filled_lines: Vec<Line> = lines
            .iter()
            .cloned()
            .chain(filler.iter().cloned())
            .collect();
        let content =
            Paragraph::new(filled_lines.into_iter().collect::<Vec<Line>>()).block(content_block);
        f.render_widget(content, area);
        viewport
    }

    fn title(&self) -> String {
        self.name.to_string()
    }
}

impl<'repo> Navigable<'repo> for BlobPager<'repo> {
    fn home(&mut self, _page_size: u16) {
        self.top = 0;
    }

    fn end(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        self.top = if !self.lines.is_empty() {
            self.lines.len() - h
        } else {
            0
        };
    }

    fn pagedown(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        let top = self.top + h;
        self.top = if top > self.lines.len() {
            self.lines.len() - 1
        } else {
            top
        }
    }

    fn pageup(&mut self, page_size: u16) {
        let h: usize = page_size.into();
        if self.top < h {
            self.top = 0;
        } else {
            self.top -= h;
        }
    }

    fn next_selection(&mut self) {
        // Always keep the last line on the screen
        if self.top < self.lines.len() - 1 {
            self.top += 1;
        }
    }

    fn previous_selection(&mut self) {
        if self.top > 0 {
            self.top -= 1;
        }
    }

    fn select(&self) -> Option<(Object<'repo>, String)> {
        None
    }

    fn selected_item(&self) -> String {
        "".to_string()
    }
}
