pub enum CurrentScreen {
    RefBrowser,
    // TreeBrowser,
    // Pager,
    Exit,
}

pub struct App {
    pub search_input: String,              // the currently being edited json key.
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
}

impl App {
    pub fn new() -> App {
        App {
            search_input: String::new(),
            current_screen: CurrentScreen::RefBrowser,
        }
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
