use std::error::Error;
use std::fmt;
use std::panic;

use color_eyre::{config::HookBuilder, eyre};

use crate::tui;

pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        tui::restore().unwrap();
        panic_hook(panic_info);
    }));

    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            tui::restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum GitBrowserError {
    Error(ErrorKind),
}

impl GitBrowserError {
    pub fn as_str(&self) -> &str {
        match &self {
            GitBrowserError::Error(err) => err.as_str(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    BinaryFileError,
    BlobReferenceError,
    SubprocessError,
    TemporaryFileError,
    TerminalInitError,
}

impl ErrorKind {
    pub fn as_str(&self) -> &str {
        match *self {
            ErrorKind::BinaryFileError => "Unable to load and display binary files",
            ErrorKind::BlobReferenceError => "Unable to load blob from repository",
            ErrorKind::SubprocessError => "Failed to execute subprocess",
            ErrorKind::TemporaryFileError => "Failed to write temporary file",
            ErrorKind::TerminalInitError => "Failed to reinitialize terminal",
        }
    }
}

impl fmt::Display for GitBrowserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GitBrowserError::Error(err) => write!(f, "{:?}", err.as_str()),
        }
    }
}

impl Error for GitBrowserError {
    fn description(&self) -> &str {
        match &self {
            GitBrowserError::Error(err) => err.as_str(),
        }
    }
}
