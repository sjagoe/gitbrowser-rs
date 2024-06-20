use git2::{Blob, Object, Repository};

use ratatui::{
    prelude::Line,
    layout::Rect,
    widgets::{
        Block,
        Paragraph,
    },
    Frame,
};

use crate::traits::{Display, Drawable, Navigable};

pub struct BlobPager<'repo> {
    top: usize,
    repo: &'repo Repository,
    blob: Blob<'repo>,
    name: String,
    lines: Vec<Line<'repo>>,
}

impl<'repo> BlobPager<'repo> {
    pub fn new(repo: &'repo Repository, blob: Blob<'repo>, name: String) -> BlobPager<'repo> {
        let content = match std::str::from_utf8(blob.content()) {
            Ok(v) => v,
            Err(e) => panic!("unable to decode utf8 {}", e),
        };
        let lines = content.lines().map(|line| Line::from(line.to_string())).collect();
        BlobPager {
            top: 0,
            repo: repo,
            blob: blob.clone(),
            name: name,
            lines: lines,
        }
    }

    pub fn from_object(repo: &'repo Repository, object: Object<'repo>, name: String) -> Self {
        match object.into_blob() {
            Ok(blob) => {
                return BlobPager::new(repo, blob, name);
            }
            Err(_) => panic!("peeling blob"),
        }
    }
}

impl<'repo> Drawable<'repo> for BlobPager<'repo> {
    fn draw(&self, f: &mut Frame, area: Rect, content_block: Block, reserved_rows: u16) {
        let viewport = 50;
        let bottom = if self.top + viewport > self.lines.len() {
            self.lines.len()
        } else {
            self.top + viewport
        };
        let lines = self.lines[self.top .. bottom].to_vec();
        let content = Paragraph::new(
            lines.into_iter().collect::<Vec<Line>>()
        ).block(content_block);
        f.render_widget(content, area);
    }

    fn title(&self) -> String {
        return format!("{}", self.name);
    }
}

impl<'repo> Navigable<'repo> for BlobPager<'repo> {
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
        return None;
    }

    fn selected_item(&self) -> String {
        return "".to_string();
    }
}
