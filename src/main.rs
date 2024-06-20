use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::Backend, Terminal};

mod app;
mod errors;
mod traits;
mod tui;
mod ui;
use crate::{
    app::{navigation::NavigationAction, App},
    ui::ui,
};

use color_eyre::Result;

use clap::Parser;
use git2::{Object, Repository};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    repository: Option<String>,

    #[arg(short, long)]
    commit_id: Option<String>,
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

    let commit: Option<Object> = match args.commit_id {
        Some(commit_id) => match repo.revparse_single(&commit_id) {
            Ok(object) => Some(object),
            Err(e) => panic!("Failed to get commit {}", e),
        },
        _ => None,
    };

    let mut terminal = tui::init()?;
    let mut app = App::new(&repo, commit);
    run_app(&mut terminal, &mut app)?;
    tui::restore()?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<bool> {
    let footer_min = 3;
    let box_border = 2;
    let reserved_height = footer_min + box_border;
    loop {
        if let Ok(frame) = terminal.draw(|f| ui(f, app, footer_min, box_border)) {
            app.set_height(frame.area.height - reserved_height);
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
                let navigation_action = NavigationAction::from(key);
                match app.navigate(navigation_action) {
                    Ok(_) => {}
                    Err(e) => app.error(e),
                }
            }
        }
    }
}
