use git2::{Blob, Object, Repository};

pub struct ExternalEditor {
    editor: String,
    name: String,
    content: Vec<u8>,
}


impl<'repo> ExternalEditor {
    pub fn new(blob: &'repo Blob, name: &str, editor: &str) -> Self {
        ExternalEditor {
            editor: editor.to_string(),
            name: name.to_string(),
            content: blob.content().to_owned(),
        }
    }
}
