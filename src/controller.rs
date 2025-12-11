use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::io;
use unicode_display_width::width;

const TAB_SIZE: u16 = 4;

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
    let mut invalid_cols = Vec::new();
    let mut curr_len = 0;
    for character in chars {
        // if character is a newline, stop building line and append it to lines
        if character == '\n' {
            lines.push(DisplayLine {
                line_content: line.iter().collect(),
                line_num,
                invalid_cols,
            });
            line = Vec::new();
            line_num += 1; // Only increment this for newline chars
            invalid_cols = Vec::new();
            curr_len = 0;
            continue;
        }

        // Handle tab spaces seperately due to dynamic sizing
        if character == '\t' {
            // Assume 4-space tabs
            let tab_len = TAB_SIZE - (curr_len % TAB_SIZE);

            // Check if tab needs to be rendered on a new line
            if curr_len + tab_len > max_line_len {
                // Doesn't fit so start a new line
                lines.push(DisplayLine {
                    line_content: line.iter().collect(),
                    line_num,
                    invalid_cols,
                });
                line = Vec::new();
                invalid_cols = Vec::new();
                curr_len = 0;
            }

            // Render tab space as TAB_SIZE number of spaces
            line.push(' ');
            for i in 1..tab_len {
                line.push(' ');
                invalid_cols.push(curr_len + 1 + i)
            }
            curr_len += tab_len;
            continue;
        }

        // Check if character needs to be rendered on a new line
        let char_width = width(&character.to_string()) as u16;
        if curr_len + char_width > max_line_len {
            // Doesn't fit so start a new line
            lines.push(DisplayLine {
                line_content: line.iter().collect(),
                line_num,
                invalid_cols,
            });
            line = Vec::new();
            invalid_cols = Vec::new();
            curr_len = 0;
        }

        line.push(character);
        for i in 1..char_width {
            invalid_cols.push(curr_len + 1 + i)
        }
        curr_len += char_width;
    }

    // Push the last line to lines
    if curr_len > 0 {
        lines.push(DisplayLine {
            line_content: line.iter().collect(),
            line_num,
            invalid_cols,
        });
    }
    return lines;
}

#[derive(Debug)]
pub struct DisplayLine {
    pub line_content: String,
    pub line_num: usize,
    pub invalid_cols: Vec<u16>, // used to ensure cursor is never in the middle of a multi-column character
}

#[derive(Debug)]
pub struct App<'a> {
    filename: &'a str,                 // Name of the file opened
    display_string: &'a str, // TEMP until buffer implemented. All references to this should be replaced by references to buffer.
    display_content: Vec<DisplayLine>, // Vector of DisplayLine structs representing content being displayed + useful info
    first_line_num: usize,             // What is the infile line number of the first line loaded?
    first_char_ind: usize, // What is the infile character index of the first character loaded?
    scroll_amount: u16,    // How far did we scroll down display_content?
    mode: Mode,
    ui_display: Vec<char>,  // Input taken from user for commands or searching
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
            first_line_num: 1,
            first_char_ind: 0,
            scroll_amount: 0,
            mode: Mode::Normal,
            ui_display: vec![],
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
    pub fn get_first_line_num(&self) -> usize {
        return self.first_line_num;
    }
    pub fn get_first_char_ind(&self) -> usize {
        return self.first_char_ind;
    }
    pub fn get_scroll_amount(&self) -> u16 {
        return self.scroll_amount;
    }
    pub fn get_mode(&self) -> &str {
        match &self.mode {
            Mode::Normal => return "Normal Mode [i]=>Insert [:]=>Command [/]=>Search",
            Mode::Command => return "Command Mode [ENTER]=>Submit [ESC]=>Exit",
            Mode::SearchInput => return "Search Mode [ENTER]=>Submit [ESC]=>Exit",
            Mode::Search => return "Search Mode [n]=>Next [p]=>Prev [ESC]=>Exit",
            Mode::Insert => return "Insertion Mode [ESC]=>Exit",
        }
    }
    pub fn get_ui_display(&self) -> String {
        return self.ui_display.iter().collect();
    }
    pub fn get_cursor_pos(&self) -> (u16, u16) {
        return self.cursor_pos;
    }
    pub fn get_term_size(&self) -> (u16, u16) {
        return self.term_size;
    }
    pub fn term_top_cursor_bound(&self) -> u16 {
        return 1;
    }
    pub fn term_bottom_cursor_bound(&self) -> u16 {
        // -4 because borders + bottom status window
        return self.term_size.0 - 4;
    }
    pub fn term_left_cursor_bound(&self) -> u16 {
        return 1;
    }
    pub fn term_right_cursor_bound(&self) -> u16 {
        // - 2 because borders
        return self.term_size.1 - 2;
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
            string_to_lines(self.display_string, term_width - 2, self.first_line_num);

        // Update cursor position if terminal size shrunk
        if self.cursor_pos.1 > self.term_right_cursor_bound() {
            // TO DO: Check if in insertion mode
            self.cursor_pos.1 = self.term_right_cursor_bound();
        }
        if self.cursor_pos.0 > self.term_bottom_cursor_bound() {
            self.cursor_pos.0 = self.term_bottom_cursor_bound();
        }

        // Edge case if enlarging terminal window and unwrapping displayed text reduced number of rows occupied by text
        if self.cursor_pos.0 > self.display_content.len() as u16 {
            // TO DO: Attempt to scroll and/or load into buffer before giving up and shifting cursor back up
            self.cursor_pos.0 = self.display_content.len() as u16;
        }

        // Ensure the cursor stays in a valid location
        self.snap_cursor();
        self.slip_cursor();
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
        match self.mode {
            Mode::Command => self.command_handle_key_event(key_event),
            Mode::Insert => self.insert_handle_key_event(key_event),
            Mode::Normal => self.normal_handle_key_event(key_event),
            Mode::SearchInput => self.search_input_handle_key_event(key_event),
            Mode::Search => self.search_handle_key_event(key_event),
        }
    }

    fn normal_handle_key_event(&mut self, key_event: KeyEvent) {
        // Clear any error/status messages once the user makes an input
        self.ui_display = vec![];
        match key_event.code {
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.ui_display = vec![':'];
            }
            KeyCode::Char('/') => {
                self.mode = Mode::SearchInput;
                self.ui_display = vec!['/'];
            }
            KeyCode::Up => self.cursor_up(),
            KeyCode::Down => self.cursor_down(),
            KeyCode::Left => self.cursor_left(),
            KeyCode::Right => self.cursor_right(),
            _ => {}
        };
    }

    fn command_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.ui_display = vec![];
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                let command: String = self.ui_display.iter().collect();
                match command.as_str() {
                    ":w" | ":W" | ":write" => { /* TO DO */ }
                    ":q" | ":Q" | ":quit" => self.exit(),
                    ":wq" | ":WQ" => { /* TO DO */ }
                    ":set num" | ":set nu" | ":num" | ":nu" => {}
                    _ => {
                        let error_msg = String::from("Error: Invalid Command");
                        self.ui_display = error_msg.chars().collect();
                    }
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.ui_display.pop();
                if self.ui_display.len() == 0 {
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Char(character) => self.ui_display.push(character),
            _ => {}
        }
    }

    fn insert_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.ui_display = vec![];
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => { /* TO DO */ }
            KeyCode::Enter => { /* TO DO */ }
            KeyCode::Tab => { /* TO DO */ }
            KeyCode::Char(character) => { /* TO DO */ }
            _ => {}
        }
    }

    fn search_input_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.ui_display = vec![];
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                // TO DO: Actually perform the search. Only transition to search mode if matches found
                self.mode = Mode::Search
            }
            KeyCode::Backspace => {
                self.ui_display.pop();
                if self.ui_display.len() == 0 {
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Char(character) => self.ui_display.push(character),
            _ => {}
        }
    }

    fn search_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.ui_display = vec![];
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') => { /* TO DO*/ }
            KeyCode::Char('p') => { /*TO DO*/ }
            _ => {}
        }
    }

    fn scroll_up(&mut self) -> Result<(), &str> {
        if self.scroll_amount > 0 {
            self.scroll_amount -= 1;
            return Ok(());
        } else {
            // TO DO: Ask buffer to read in another line from rope, then push out a line from other end
            return Err("Error: Start of file reached");
        }
    }

    fn scroll_down(&mut self) -> Result<(), &str> {
        let max_scroll_amount = self.display_content.len()
            - (self.term_bottom_cursor_bound() - self.term_top_cursor_bound() + 1) as usize;
        if (self.scroll_amount as usize) < max_scroll_amount {
            self.scroll_amount += 1;
            return Ok(());
        } else {
            // TO DO: Ask buffer to read in another line from rope, then push out a line from other end
            return Err("Error: End of file reached");
        }
    }

    fn cursor_up(&mut self) {
        if self.cursor_pos.0 > self.term_top_cursor_bound() {
            self.cursor_pos.0 -= 1;

            // Snap cursor to end of line after moving down
            self.snap_cursor();

            // If cursor just moved into the middle of a wide character (ex tab space) 'slip' it leftwards to valid space
            self.slip_cursor();
        } else {
            match self.scroll_up() {
                Ok(_) => {}
                Err(msg) => self.ui_display = msg.chars().collect(),
            };
        }
    }

    fn cursor_down(&mut self) {
        // Edge case: small file, big terminal. End of file was reached
        if (self.cursor_pos.0 as usize) == self.display_content.len() {
            self.ui_display = "Error: End of file reached".chars().collect();
            return;
        }

        if self.cursor_pos.0 < self.term_bottom_cursor_bound() {
            self.cursor_pos.0 += 1;

            // Snap cursor to end of line after moving down
            self.snap_cursor();

            // If cursor just moved into the middle of a wide character (ex tab space) 'slip' it leftwards to valid space
            self.slip_cursor();
        } else {
            match self.scroll_down() {
                Ok(_) => {}
                Err(msg) => self.ui_display = msg.chars().collect(),
            };
        }
    }

    fn cursor_right(&mut self) {
        // display_content is 0-indexed, cursor_pos is 1-indexed
        let line = &self.display_content[self.cursor_pos.0 as usize - 1];

        // If cursor will move into the middle of a wide character (ex tab space) 'slip' it rightwards until the next character is valid
        let invalid_cols = &line.invalid_cols;
        while invalid_cols.contains(&(self.cursor_pos.1 + 1)) {
            self.cursor_pos.1 += 1;
        }

        // If the cursor is at the end of the line, move to the start of the next line if available
        if self.cursor_pos.1 as u64 >= width(&line.line_content) {
            // Edge case: small file, big terminal. End of file was reached
            if (self.cursor_pos.0 as usize) == self.display_content.len() {
                self.ui_display = "Error: End of file reached".chars().collect();
                return;
            }

            // If scrolling needed, try to do so
            if self.cursor_pos.0 == self.term_bottom_cursor_bound() {
                match self.scroll_down() {
                    Ok(_) => {}
                    Err(msg) => {
                        self.ui_display = msg.chars().collect();
                        return;
                    }
                };
            }
            self.cursor_pos.0 += 1;
            self.cursor_pos.1 = 1;
        } else {
            self.cursor_pos.1 += 1;
        }
    }

    fn cursor_left(&mut self) {
        // If the cursor is at the start of the line, move to the end of the previous line if available
        if self.cursor_pos.1 == self.term_left_cursor_bound() {
            // If scrolling needed, try to do so
            if self.cursor_pos.0 == self.term_top_cursor_bound() {
                match self.scroll_up() {
                    Ok(_) => {}
                    Err(msg) => {
                        self.ui_display = msg.chars().collect();
                        return;
                    }
                };
            }

            self.cursor_pos.0 -= 1;
            // display_content is 0-indexed, cursor_pos is 1-indexed
            let line = &self.display_content[self.cursor_pos.0 as usize - 1].line_content;
            self.cursor_pos.1 = width(line) as u16;
        } else {
            self.cursor_pos.1 -= 1;
        }

        // If cursor just moved into the middle of a wide character (ex tab space) 'slip' it leftwards to valid space
        self.slip_cursor();
    }

    fn snap_cursor(&mut self) {
        // display_content is 0-indexed, cursor_pos is 1-indexed
        let line = &self.display_content[self.cursor_pos.0 as usize - 1].line_content;

        // Snap cursor to end of line after moving up
        let line_len = width(line) as u16;
        if line_len < self.cursor_pos.1 {
            self.cursor_pos.1 = line_len;
        }
    }

    fn slip_cursor(&mut self) {
        // display_content is 0-indexed, cursor_pos is 1-indexed
        let invalid_cols = &self.display_content[self.cursor_pos.0 as usize - 1].invalid_cols;

        // If cursor just moved into the middle of a wide character (ex tab space) 'slip' it leftwards to valid space
        while invalid_cols.contains(&self.cursor_pos.1) {
            self.cursor_pos.1 -= 1;
        }
    }
}
