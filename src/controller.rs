use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Size;
use std::cmp;
use std::io;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Command,
    SearchInput,
    Search,
    Insert,
}

#[derive(Debug)]
pub struct App<'a> {
    filename: &'a str,        // Name of the file opened
    display_content: &'a str, // str slice representing the section of text currently being displayed in the View UI
    file_line: usize,         // Which line of the file appears at the top of the terminal window
    mode: Mode,
    ui_display: String,     // Input taken from user for commands or searching
    cursor_pos: (u16, u16), // cursor position in terminal. (y, x), or (row, col), with 1,1 being the top-left corner (1 not 0 due to border)
    term_size: (u16, u16),
    running: bool,
}

impl<'a> App<'a> {
    pub fn new(
        filename: &'a mut str,
        display_content: &'a mut str,
        term_height: u16,
        term_width: u16,
    ) -> Self {
        Self {
            filename,
            display_content,
            file_line: 1,
            mode: Mode::Normal,
            ui_display: String::from(""),
            cursor_pos: (1, 1),
            term_size: (term_height, term_width),
            running: true,
        }
    }

    pub fn get_filename(&self) -> &str {
        return self.filename;
    }
    pub fn get_content(&self) -> &str {
        return self.display_content;
    }
    pub fn get_fileline(&self) -> usize {
        return self.file_line;
    }
    pub fn get_mode(&self) -> &str {
        match &self.mode {
            Mode::Normal => return "Normal Mode",
            Mode::Command => return "Command Mode",
            Mode::SearchInput => return "Search Mode",
            Mode::Search => return "Search Mode",
            Mode::Insert => return "Insertion Mode",
        }
    }
    pub fn get_ui_display(&self) -> &str {
        return &self.ui_display;
    }
    pub fn get_cursor_pos(&self) -> (u16, u16) {
        return self.cursor_pos;
    }
    pub fn running(&self) -> bool {
        return self.running;
    }

    fn exit(&mut self) {
        self.running = false;
    }

    pub fn update_term_size(&mut self, term_height: u16, term_width: u16) {
        self.term_size = (term_height, term_width);
    }

    /*
     * Accepts any user inputs provided via crossterm while the program is running,
     * and passes them to the Controller for further processing
     */
    pub fn handle_events(&mut self) -> io::Result<()> {
        // TO DO: event::read is a blocking call, consider using event::poll instead?
        match event::read()? {
            // Checks that this was a key press event.
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    self.handle_key_event(key_event)
                }
            }
            _ => {}
        };
        return Ok(());
    }

    /*
     * Handles key press events specifically
     */
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(), // Temp exit command until the controller is implemented
            KeyCode::Up => self.cursor_up(),
            KeyCode::Down => self.cursor_down(),
            KeyCode::Left => self.cursor_left(),
            KeyCode::Right => self.cursor_right(),
            _ => { /* To be implemented */ }
        };
    }

    fn cursor_up(&mut self) {
        if self.cursor_pos.0 > 1 {
            self.cursor_pos.0 = self.cursor_pos.0 - 1;
        } else {
            // Scroll content upwards if available
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor_pos.0 < self.term_size.0 - 4 {
            self.cursor_pos.0 = self.cursor_pos.0 + 1;
        }
    }

    fn cursor_right(&mut self) {
        self.cursor_pos.1 = cmp::min(self.cursor_pos.1 + 1, self.term_size.1 - 2);
    }

    fn cursor_left(&mut self) {
        self.cursor_pos.1 = cmp::max(self.cursor_pos.1 - 1, 1); // TO DO: Update when line numbers implemented
    }
}
