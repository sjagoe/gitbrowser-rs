use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn ui(f: &mut Frame, app: &App, footer_min: u16, box_border: u16) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(box_border + 2),
            Constraint::Length(footer_min),
        ])
        .split(f.size());

    app.draw(f, chunks[0], footer_min + box_border);

    let current_keys_hint = Span::styled("(^x) exit | (^g) back", Style::default().fg(Color::Red));

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    f.render_widget(key_notes_footer, chunks[1]);

    // if let Some(editing) = &app.currently_editing {
    //     let popup_block = Block::default()
    //         .title("Enter a new key-value pair")
    //         .borders(Borders::NONE)
    //         .style(Style::default().bg(Color::DarkGray));

    //     let area = centered_rect(60, 25, f.size());
    //     f.render_widget(popup_block, area);

    //     let popup_chunks = Layout::default()
    //         .direction(Direction::Horizontal)
    //         .margin(1)
    //         .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
    //         .split(area);

    //     let mut key_block = Block::default().title("Key").borders(Borders::ALL);
    //     let mut value_block = Block::default().title("Value").borders(Borders::ALL);

    //     let active_style = Style::default().bg(Color::LightYellow).fg(Color::Black);

    //     match editing {
    //         CurrentlyEditing::Key => key_block = key_block.style(active_style),
    //         CurrentlyEditing::Value => value_block = value_block.style(active_style),
    //     };

    //     let key_text = Paragraph::new(app.key_input.clone()).block(key_block);
    //     f.render_widget(key_text, popup_chunks[0]);

    //     let value_text = Paragraph::new(app.value_input.clone()).block(value_block);
    //     f.render_widget(value_text, popup_chunks[1]);
    // }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
