use std::io::Write as _;
use std::process::Command;

use git2::Blob;

use tempfile::Builder;

use crate::tui;

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

    pub fn display(&self) {
        match Builder::new().suffix(&self.name).tempfile() {
            Ok(mut tempfile) => {
                tui::restore().expect("failed to restore terminal");
                eprintln!("Opening {} with {} ...", self.name, self.editor);
                let file = tempfile.as_file_mut();
                file.write_all(&self.content).expect("failed to write file");

                let mut emacsclient = Command::new(&self.editor)
                    .arg(tempfile.path())
                    .spawn()
                    .expect("failed to run emacsclient");

                let ecode = emacsclient
                    .wait()
                    .expect("failed waiting for emacsclient to exit");

                assert!(ecode.success());
                tui::init().expect("failed to reinit terminal");
            }
            // We should use a handlable error
            Err(_) => panic!("unable to create temporary file"),
        }
    }
}
