use git2::Repository;

use ratatui::{
    prelude::Modifier,
    style::{Color, Style},
    text::{Line, Span},
    layout::Rect,
    widgets::{
        block::{Padding, Title},
        Block, Borders, List, ListItem, Paragraph, Widget,
    },
    Frame,
};

pub enum CurrentScreen {
    RefBrowser,
    TreeBrowser,
    // Pager,
    // Exit,
}

struct RefsPage<'repo> {
    repo: &'repo Repository,
    selected_index: usize,
}

pub struct App<'repo> {
    pub selected_index: usize,
    pub search_input: String,
    pub current_screen: CurrentScreen,
    refs_page: RefsPage<'repo>,
}

impl<'repo> RefsPage<'repo> {
    pub fn new(repo: &'repo Repository) -> RefsPage<'repo> {
        RefsPage {
            repo: repo,
            selected_index: 0,
        }
    }

    fn items(&self) -> Vec<String> {
        let mut refs = match self.repo.references() {
            Ok(r) => r,
            Err(_e) => return vec![],
        };
        return refs.names().map(|refname| refname.unwrap().to_string()).collect();
    }

    fn draw(&self, f: &mut Frame, area: Rect, content_block: Block, reserved_rows: u16) {
        let mut list_items = Vec::<ListItem>::new();
        let items = self.items();

        let visible = f.size().height - reserved_rows;
        let (_page, _pages, page_start_index) = pagination(items.len(), visible.into(), self.selected_index);

        let end_slice = if page_start_index + usize::from(visible) >= items.len() {
            items.len()
        } else {
            page_start_index + usize::from(visible)
        };
        let display_items = &items[page_start_index .. end_slice];

        for (pos, item) in display_items.iter().enumerate() {
            let style = if pos + page_start_index == self.selected_index {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };
            list_items.push(ListItem::new(Line::from(Span::styled(item, style))));
        }
        let content = List::new(list_items).block(content_block);
        f.render_widget(content, area);
    }

    fn title(&self) -> String {
        if let Some(path) = self.repo.path().parent() {
            if let Some(name) = path.file_name() {
                return format!("{}", name.to_string_lossy());
            } else {
                return format!("{}", path.to_string_lossy());
            }
        } else {
            return format!("{}", self.repo.path().to_string_lossy());
        };
    }

    fn next_selection(&mut self) {
        if self.selected_index < self.items().len() - 1 {
            self.selected_index += 1;
        }
    }

    fn previous_selection(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn select(&mut self) {}

    fn back(&mut self) {}
}


impl<'repo> App<'repo> {
    pub fn new(repo: &'repo Repository) -> App<'repo> {
        App {
            selected_index: 0,
            search_input: String::new(),
            current_screen: CurrentScreen::RefBrowser,
            refs_page: RefsPage::new(repo)
        }
    }

    pub fn title(&self) -> Vec<Span> {
        let mut parts = vec![
            Span::from(" "),
        ];
        parts.push(
            Span::styled(
                self.refs_page.title(),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        );
        parts.push(Span::from(" "));
        return parts;
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, reserved_rows: u16) {
        let title = Title::from(self.title());
        let content_block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .style(Style::default())
            .title(title);

        self.refs_page.draw(f, area, content_block, reserved_rows);
    }

    pub fn next_selection(&mut self) {
        self.refs_page.next_selection();
    }

    pub fn previous_selection(&mut self) {
        self.refs_page.previous_selection();
    }

    pub fn select(&mut self) {
        self.refs_page.select();
    }

    pub fn back(&mut self) {
        self.refs_page.back();
    }

    // pub fn save_key_value(&mut self) {
    //     self.pairs
    //         .insert(self.key_input.clone(), self.value_input.clone());

    //     self.key_input = String::new();
    //     self.value_input = String::new();
    //     self.currently_editing = None;
    // }

    // pub fn toggle_editing(&mut self) {
    //     if let Some(edit_mode) = &self.currently_editing {
    //         match edit_mode {
    //             CurrentlyEditing::Key => self.currently_editing = Some(CurrentlyEditing::Value),
    //             CurrentlyEditing::Value => self.currently_editing = Some(CurrentlyEditing::Key),
    //         };
    //     } else {
    //         self.currently_editing = Some(CurrentlyEditing::Key);
    //     }
    // }
}

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
