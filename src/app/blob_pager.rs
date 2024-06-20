use git2::{Blob, Object, Repository};

use ratatui::{
    layout::Rect,
    widgets::{
        Block,
        Paragraph,
    },
    Frame,
};

use crate::traits::{Display, Drawable, Navigable};

pub struct BlobPager<'repo> {
    repo: &'repo Repository,
    blob: Blob<'repo>,
    name: String,
}

impl<'repo> BlobPager<'repo> {
    pub fn new(repo: &'repo Repository, blob: Blob<'repo>, name: String) -> BlobPager<'repo> {
        BlobPager {
            repo: repo,
            blob: blob,
            name: name,
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
        let content = Paragraph::new("data").block(content_block);
        f.render_widget(content, area);
    }

    fn title(&self) -> String {
        return format!("{}", self.name);
    }
}
