use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod errors;
mod tui;
mod app;
mod ui;
use crate::{
    app::{App, CurrentScreen},
    ui::ui,
};

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};

use clap::Parser;
use git2::Repository;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    repository: Option<String>,
}

fn main() -> Result<()> {
    errors::install_hooks()?;

    let args = Args::parse();
    let repo_path = match args.repository {
        Some(repo) => repo,
        _ => ".".to_string(),
    };

    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let mut terminal = tui::init()?;
    let mut app = App::new(repo);
    run_app(&mut terminal, &mut app)?;
    tui::restore()?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::RefBrowser => match key.code {
                    // KeyCode::Char('e') => {
                    //     app.current_screen = CurrentScreen::Editing;
                    // }
                    KeyCode::Char('q') => {
                        return Ok(true);
                    }
                    _ => {}
                },
                // CurrentScreen::Editing if key.kind == KeyEventKind::Press => {
                //     match key.code {
                //         KeyCode::Enter => {
                //             if let Some(editing) = &app.currently_editing {
                //                 match editing {
                //                     CurrentlyEditing::Key => {
                //                         app.currently_editing = Some(CurrentlyEditing::Value);
                //                     }
                //                     CurrentlyEditing::Value => {
                //                         app.save_key_value();
                //                         app.current_screen = CurrentScreen::Main;
                //                     }
                //                 }
                //             }
                //         }
                //         KeyCode::Backspace => {
                //             if let Some(editing) = &app.currently_editing {
                //                 match editing {
                //                     CurrentlyEditing::Key => {
                //                         app.key_input.pop();
                //                     }
                //                     CurrentlyEditing::Value => {
                //                         app.value_input.pop();
                //                     }
                //                 }
                //             }
                //         }
                //         KeyCode::Esc => {
                //             app.current_screen = CurrentScreen::Main;
                //             app.currently_editing = None;
                //         }
                //         KeyCode::Tab => {
                //             app.toggle_editing();
                //         }
                //         KeyCode::Char(value) => {
                //             if let Some(editing) = &app.currently_editing {
                //                 match editing {
                //                     CurrentlyEditing::Key => {
                //                         app.key_input.push(value);
                //                     }
                //                     CurrentlyEditing::Value => {
                //                         app.value_input.push(value);
                //                     }
                //                 }
                //             }
                //         }
                //         _ => {}
                //     }
                // }
                _ => {}
            }
        }
    }
}
