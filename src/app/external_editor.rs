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

        let mut command = match Command::new(&self.editor).arg(tempfile.path()).spawn() {
            Ok(command) => command,
            Err(_) => {
                return Err(GitBrowserError::Error(ErrorKind::SubprocessError));
            }
        };

        let status = match command.wait() {
            Ok(status) => status,
            Err(_) => {
                return Err(GitBrowserError::Error(ErrorKind::SubprocessError));
            }
        };

        if !status.success() {
            return Err(GitBrowserError::Error(ErrorKind::SubprocessError));
        }
        Ok(())
    }

    pub fn display(&self) -> Result<(), GitBrowserError> {
        match Builder::new().suffix(&self.name).tempfile() {
            Ok(mut tempfile) => {
                if tui::restore().is_err() {
                    return Err(GitBrowserError::Error(ErrorKind::TerminalInitError));
                }
                eprintln!("Opening {} with {} ...", self.name, self.editor);

                match self.spawn_editor(&mut tempfile) {
                    Ok(_) => {}
                    Err(e) => {
                        if tui::init().is_err() {
                            return Err(GitBrowserError::Error(ErrorKind::TerminalInitError));
                        }
                        return Err(e);
                    }
                }

                if tui::init().is_err() {
                    return Err(GitBrowserError::Error(ErrorKind::TerminalInitError));
                }
            }
            Err(_) => {
                return Err(GitBrowserError::Error(ErrorKind::TemporaryFileError));
            }
        }
        Ok(())
    }
}
