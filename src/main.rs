use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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
    app::{App, CurrentScreen, CurrentlyEditing},
    ui::ui,
};

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};

fn main() -> Result<()> {
    errors::install_hooks()?;
    let mut terminal = tui::init()?;
    let mut app = App::new();
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
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('e') => {
                        app.current_screen = CurrentScreen::Editing;
                        app.currently_editing = Some(CurrentlyEditing::Key);
                    }
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('q') => {
                        return Ok(false);
                    }
                    _ => {}
                },
                CurrentScreen::Editing if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(editing) = &app.currently_editing {
                                match editing {
                                    CurrentlyEditing::Key => {
                                        app.currently_editing = Some(CurrentlyEditing::Value);
                                    }
                                    CurrentlyEditing::Value => {
                                        app.save_key_value();
                                        app.current_screen = CurrentScreen::Main;
                                    }
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if let Some(editing) = &app.currently_editing {
                                match editing {
                                    CurrentlyEditing::Key => {
                                        app.key_input.pop();
                                    }
                                    CurrentlyEditing::Value => {
                                        app.value_input.pop();
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::Main;
                            app.currently_editing = None;
                        }
                        KeyCode::Tab => {
                            app.toggle_editing();
                        }
                        KeyCode::Char(value) => {
                            if let Some(editing) = &app.currently_editing {
                                match editing {
                                    CurrentlyEditing::Key => {
                                        app.key_input.push(value);
                                    }
                                    CurrentlyEditing::Value => {
                                        app.value_input.push(value);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
