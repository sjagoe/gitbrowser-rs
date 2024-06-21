use std::io::Write as _;
use std::process::Command;

use color_eyre::Result;

use git2::Blob;

use tempfile::{Builder, NamedTempFile};

use crate::{
    errors::{ErrorKind, GitBrowserError},
    tui,
};

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

    fn spawn_editor(&self, tempfile: &mut NamedTempFile) -> Result<(), GitBrowserError> {
        let file = tempfile.as_file_mut();
        file.write_all(&self.content).expect("failed to write file");

        let mut command = Command::new(&self.editor)
            .arg(tempfile.path())
            .spawn()
            .map_err(|_| GitBrowserError::Error(ErrorKind::SubprocessError))?;

        let status = command
            .wait()
            .map_err(|_| GitBrowserError::Error(ErrorKind::SubprocessError))?;

        if status.success() {
            Ok(())
        } else {
            Err(GitBrowserError::Error(ErrorKind::SubprocessError))
        }
    }

    pub fn display(&self) -> Result<(), GitBrowserError> {
        let mut tempfile = Builder::new()
            .suffix(&self.name)
            .tempfile()
            .map_err(|_| GitBrowserError::Error(ErrorKind::TemporaryFileError))?;

        tui::restore().map_err(|_| GitBrowserError::Error(ErrorKind::TerminalInitError))?;
        eprintln!("Opening {} with {} ...", self.name, self.editor);

        let error = self.spawn_editor(&mut tempfile).err();
        tui::init().map_err(|_| GitBrowserError::Error(ErrorKind::TerminalInitError))?;

        if let Some(e) = error {
            return Err(e);
        }
        Ok(())
    }
}
