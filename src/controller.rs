use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::cmp;
use std::io;
use unicode_display_width::width;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Command,
    SearchInput,
    Search,
    Insert,
}

fn string_to_lines(string: &str, max_line_len: u16, first_line_num: usize) -> Vec<DisplayLine> {
    let chars = string.chars();
    let mut lines: Vec<DisplayLine> = Vec::new();
    let mut line: Vec<char> = Vec::new();
    let mut line_num = first_line_num;
    let mut char_widths = Vec::new();
    let mut invalid_cols = Vec::new();
    let mut curr_len = 0;
    for character in chars {
        // if character is a newline, stop building line and append it to lines
        if character == '\n' {
            lines.push(DisplayLine {
                line_content: line.iter().collect(),
                line_num,
                char_widths,
                invalid_cols,
            });
            line = Vec::new();
            line_num += 1; // Only increment this for newline chars
            char_widths = Vec::new();
            invalid_cols = Vec::new();
            curr_len = 0;
            continue;
        }

        // Handle tab spaces seperately due to dynamic sizing
        if character == '\t' {
            // Assume 4-space tabs
            let tab_len = 4 - (curr_len % 4);

            // Check if tab needs to be rendered on a new line
            if curr_len + tab_len > max_line_len {
                // Doesn't fit so start a new line
                lines.push(DisplayLine {
                    line_content: line.iter().collect(),
                    line_num,
                    char_widths,
                    invalid_cols,
                });
                line = Vec::new();
                char_widths = Vec::new();
                invalid_cols = Vec::new();
                curr_len = 0;
            }

            // Render tab space as tab_len number of spaces
            line.push(' ');
            for i in 1..tab_len {
                line.push(' ');
                invalid_cols.push(curr_len + 1 + i)
            }
            char_widths.push(tab_len);
            curr_len = curr_len + tab_len;
            continue;
        }

        // Check if character needs to be rendered on a new line
        let char_width = width(&character.to_string()) as u16;
        if curr_len + char_width > max_line_len {
            // Doesn't fit so start a new line
            lines.push(DisplayLine {
                line_content: line.iter().collect(),
                line_num,
                char_widths,
                invalid_cols,
            });
            line = Vec::new();
            char_widths = Vec::new();
            invalid_cols = Vec::new();
            curr_len = 0;
        }

        line.push(character);
        char_widths.push(char_width);
        for i in 1..char_width {
            invalid_cols.push(curr_len + 1 + i)
        }
        curr_len = curr_len + char_width;
    }

    // Push the last line to lines
    if curr_len > 0 {
        lines.push(DisplayLine {
            line_content: line.iter().collect(),
            line_num,
            char_widths,
            invalid_cols,
        });
    }
    return lines;
}

#[derive(Debug)]
pub struct DisplayLine {
    pub line_content: String,
    pub line_num: usize,
    pub char_widths: Vec<u16>, // used for left/right stepping and end-of-line snapping
    pub invalid_cols: Vec<u16>, // used to ensure cursor is never in the middle of a multi-column character
}

#[derive(Debug)]
pub struct App<'a> {
    filename: &'a str,                 // Name of the file opened
    display_string: &'a str, // TEMP until buffer implemented. All references to this should be replaced by references to buffer.
    display_content: Vec<DisplayLine>, // Vector of DisplayLine structs representing content being displayed + useful info
    display_file_line: usize, // Which line of the file corresponds to the first line of text loaded into display_content
    scroll_amount: usize,     // How far did we scroll down display_content?
    mode: Mode,
    ui_display: String,     // Input taken from user for commands or searching
    cursor_pos: (u16, u16), // cursor position in terminal. (y, x), or (row, col), with 1,1 being the top-left corner (1 not 0 due to border)
    term_size: (u16, u16),  // Terminal size
    running: bool,
}

impl<'a> App<'a> {
    pub fn new(
        filename: &'a mut str,
        display_string: &'a mut str,
        term_height: u16,
        term_width: u16,
    ) -> Self {
        Self {
            filename,
            display_string,
            display_content: string_to_lines(display_string, term_width - 2, 1),
            display_file_line: 1,
            scroll_amount: 0,
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
    pub fn get_content(&self) -> &Vec<DisplayLine> {
        return &self.display_content;
    }
    pub fn get_display_fileline(&self) -> usize {
        return self.display_file_line;
    }
    pub fn get_scroll_amount(&self) -> usize {
        return self.scroll_amount;
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
    pub fn get_term_size(&self) -> (u16, u16) {
        return self.term_size;
    }
    pub fn running(&self) -> bool {
        return self.running;
    }

    fn exit(&mut self) {
        self.running = false;
    }

    pub fn update_term_size(&mut self, term_height: u16, term_width: u16) {
        self.term_size = (term_height, term_width);
        // TEMP: Future should use ref to buffer instead of display_string
        self.display_content =
            string_to_lines(self.display_string, term_width - 2, self.display_file_line);

        // Update cursor position if terminal size shrunk
        if self.cursor_pos.1 > self.term_size.1 - 2 {
            // TO DO: Check if in insertion mode
            self.cursor_pos.1 = self.term_size.1 - 2;
        }
        // TO DO: Snap cursor to end of line
        if self.cursor_pos.0 > self.term_size.0 - 4 {
            self.cursor_pos.0 = self.term_size.0 - 4;
        }
        // TO DO: Handle edge case where expand term width -> last line gets unwrapped -> cursor now outside of text
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
            Event::Resize(col, row) => self.update_term_size(row, col),
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
            let new_line = &self.display_content[self.cursor_pos.0 as usize - 1];

            // Snap cursor to end of line after moving up
            let line_len = width(&new_line.line_content) as u16;
            if line_len < self.cursor_pos.1 {
                self.cursor_pos.1 = line_len;
            }

            // If cursor just moved into the middle of a wide character (ex tab space) 'slip' it leftwards to valid space
            let invalid_cols = &new_line.invalid_cols;
            while invalid_cols.contains(&self.cursor_pos.1) {
                self.cursor_pos.1 -= 1;
            }
        } else {
            // TO DO: Scroll content upwards if available
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor_pos.0 < self.term_size.0 - 4 {
            self.cursor_pos.0 = self.cursor_pos.0 + 1;
            let new_line = &self.display_content[self.cursor_pos.0 as usize - 1];

            // Snap cursor to end of line after moving down
            let line_len = width(&new_line.line_content) as u16;
            if line_len < self.cursor_pos.1 {
                self.cursor_pos.1 = line_len;
            }

            // If cursor just moved into the middle of a wide character (ex tab space) 'slip' it leftwards to valid space
            let invalid_cols = &new_line.invalid_cols;
            while invalid_cols.contains(&self.cursor_pos.1) {
                self.cursor_pos.1 -= 1;
            }
        } else {
            // TO DO: Scroll content upwards if available
        }
    }

    fn cursor_right(&mut self) {
        self.cursor_pos.1 = cmp::min(self.cursor_pos.1 + 1, self.term_size.1 - 2);
    }

    fn cursor_left(&mut self) {
        self.cursor_pos.1 = cmp::max(self.cursor_pos.1 - 1, 1);
    }
}
