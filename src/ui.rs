use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        block::{Padding, Title},
        Block, Borders, List, ListItem, Paragraph,
    },
    Frame,
};

use crate::app::{App};

fn pagination(item_count: usize, visible_item_count: usize, selected_index: usize) -> (usize, usize, usize) {
    let page_start_index = selected_index - (selected_index % visible_item_count);
    let pages = if item_count % visible_item_count > 0 {
        item_count / visible_item_count + 1
    } else {
        item_count / visible_item_count
    };
    let page = if page_start_index % visible_item_count > 0 {
        page_start_index / visible_item_count + 1
    } else {
        page_start_index / visible_item_count
    };
    (page, pages, page_start_index)
}

pub fn ui(f: &mut Frame, app: &App) {
    // Create the layout sections.
    let footer_min = 3;
    let box_border = 2;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(box_border + 2),
            Constraint::Length(footer_min),
        ])
        .split(f.size());

    let title = Title::from(app.title());
    let content_block = Block::default()
        .padding(Padding::horizontal(1))
        .borders(Borders::ALL)
        .style(Style::default())
        .title(title);

    let mut list_items = Vec::<ListItem>::new();
    let items = app.items();

    let visible = f.size().height - footer_min - box_border;
    let (_page, _pages, page_start_index) = pagination(items.len(), visible.into(), app.selected_index);

    let end_slice = if page_start_index + usize::from(visible) >= items.len() {
        items.len()
    } else {
        page_start_index + usize::from(visible)
    };
    let display_items = &items[page_start_index .. end_slice];

    for (pos, item) in display_items.iter().enumerate() {
        let style = if pos + page_start_index == app.selected_index {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };
        list_items.push(ListItem::new(Line::from(Span::styled(item, style))));
    }

    let list = List::new(list_items).block(content_block);

    f.render_widget(list, chunks[0]);
    let current_keys_hint = {
        match app.current_screen {
            _ => Span::styled(
                "(^x) exit | (^g) back",
                Style::default().fg(Color::Red),
            ),
        }
    };

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

// /// helper function to create a centered rect using up certain percentage of the available rect `r`
// fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
//     // Cut the given rectangle into three vertical pieces
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Percentage((100 - percent_y) / 2),
//             Constraint::Percentage(percent_y),
//             Constraint::Percentage((100 - percent_y) / 2),
//         ])
//         .split(r);

//     // Then cut the middle vertical piece into three width-wise pieces
//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([
//             Constraint::Percentage((100 - percent_x) / 2),
//             Constraint::Percentage(percent_x),
//             Constraint::Percentage((100 - percent_x) / 2),
//         ])
//         .split(popup_layout[1])[1] // Return the middle chunk
// }
