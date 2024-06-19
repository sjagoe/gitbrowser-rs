use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use ratatui::{
    backend::{Backend},
    Terminal,
};

mod errors;
mod tui;
mod app;
mod ui;
use crate::{
    app::{App},
    ui::ui,
};

use color_eyre::{
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
    let mut app = App::new(&repo);
    run_app(&mut terminal, &mut app)?;
    tui::restore()?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        let read_event = event::read()?;
        // Global keys
        if let Event::Key(key) = read_event {
            if key.kind == KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            if key.code == KeyCode::Char('x') && key.modifiers == KeyModifiers::CONTROL {
                return Ok(true);
            }
            if key.code == KeyCode::Char('g') && key.modifiers == KeyModifiers::CONTROL {
                app.back();
                continue;
            }
        }
        // Page-specific keys
        match read_event {
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers,
                ..
            }) =>  {
                if modifiers == KeyModifiers::empty() {
                    app.next_selection();
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers,
                ..
            }) =>  {
                if modifiers == KeyModifiers::empty() {
                    app.previous_selection();
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers,
                ..
            }) =>  {
                if modifiers == KeyModifiers::empty() {
                    app.select();
                }
            }
            _ => {}
        }
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
    }
}
