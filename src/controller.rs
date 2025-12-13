use crate::view::MAX_HELP_SCROLL;
use crate::model::{self, EditorModel};
use count_digits::CountDigits;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::cmp;
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
    Minimized, //Used to prevent cursor out of bounds crash when terminal is shrunk to <=4 lines tall
    Help,      // Used to display the help screen
}

fn string_to_lines(
    string: &str,
    term_width: u16,
    first_line_num: usize,
    first_char_ind: usize,
    show_lines: bool,
) -> Vec<DisplayLine> {
    let chars = string.chars();
    let max_line_len = term_width - 2; // to accomodate the two borders
    let mut lines: Vec<DisplayLine> = Vec::new();
    let mut line = DisplayLine::new(first_line_num, first_char_ind, 0);
    let mut num_chars = 0; // Used for calculating file indexing
    let mut total_char_width = 0; // Used to track how many characters can fit on a line
    if show_lines {
        line.line_content = first_line_num.to_string();
        line.line_content.push('|');
        total_char_width = first_line_num.count_digits() as u16 + 1;
    }
    for character in chars {
        // if character is a newline, stop building line and append it to lines
        if character == '\n' {
            lines.push(line);
            let last_line = &lines[lines.len() - 1];
            line = DisplayLine::new(
                last_line.line_num + 1,
                last_line.infile_index + num_chars,
                0,
            );
            num_chars = 0;
            total_char_width = 0;
            if show_lines {
                line.line_content = line.line_num.to_string();
                line.line_content.push('|');
                total_char_width = line.line_num.count_digits() as u16 + 1;
            }
            continue;
        }

        // Get the number of columns required to display this character (ex. some emoticons take 2 columns)
        let char_width = if character == '\t' {
            // Handle tab spaces seperately due to dynamic sizing
            TAB_SIZE - (total_char_width % TAB_SIZE)
        } else {
            width(&character.to_string()) as u16
        };

        // Check if character needs to be rendered on a new line
        if total_char_width + char_width > max_line_len {
            // Character doesn't fit, so start a new line
            lines.push(line);
            let last_line = &lines[lines.len() - 1];
            line = DisplayLine::new(
                last_line.line_num,
                last_line.infile_index + num_chars,
                last_line.inline_index + num_chars,
            );
            num_chars = 0;
            total_char_width = 0;
            if show_lines {
                line.line_content = line.line_num.to_string();
                line.line_content.push('|');
                total_char_width = line.line_num.count_digits() as u16 + 1;
            }
        }

        // Add character to line to be rendered
        if character == '\t' {
            // Render tab space as a set of spaces
            // while keeping track of which of them are 'invalid' for the cursor
            line.line_content.push(' ');
            num_chars += 1;
            for i in 1..char_width {
                line.line_content.push(' ');
                line.invalid_cols.push(total_char_width + 1 + i)
            }
            total_char_width += char_width;
        } else {
            line.line_content.push(character);
            num_chars += 1;
            // Keep track of 'invalid columns' for cursor due to wide characters
            for i in 1..char_width {
                line.invalid_cols.push(total_char_width + 1 + i)
            }
            total_char_width += char_width;
        }
    }

    // Push the last line to lines
    if total_char_width > 0 {
        lines.push(line);
    }
    return lines;
}

#[derive(Debug)]
pub struct DisplayLine {
    pub line_content: String,
    pub line_num: usize,
    pub infile_index: usize, // Char index of the start of this displayed line in the total file
    pub inline_index: usize, // Char index of the start of this displayed line in the file line
    pub invalid_cols: Vec<u16>, // used to ensure cursor is never in the middle of a multi-column character
}

impl DisplayLine {
    pub fn new(line_num: usize, infile_index: usize, inline_index: usize) -> Self {
        Self {
            line_content: String::new(),
            line_num,
            infile_index,
            inline_index,
            invalid_cols: vec![],
        }
    }
}

#[derive(Debug)]
pub struct App<'a> {
    model: &'a mut EditorModel,
    display_content: Vec<DisplayLine>, // Vector of DisplayLine structs representing content being displayed + useful info
    first_line_num: usize, // What is the line number of the first line loaded? Used for line number display
    first_char_ind: usize, // What is the infile character index of the first character loaded? Used for cursor indexing
    scroll_amount: u16,    // How far did we scroll down display_content?
    scroll_help_amount: u16, // How far to scroll help popup
    mode: Mode,
    show_line_nums: bool,
    msg_display: Vec<char>, // Input taken from user for commands or searching
    search_term: Option<String>, // What is being searched for in search mode. Only assigned on successful match for View highlighting
    cursor_pos: (u16, u16), // cursor position in terminal. (y, x), or (row, col), with 1,1 being the top-left corner (1 not 0 due to border)
    term_size: (u16, u16),  // Terminal size
    running: bool,
}

impl<'a> App<'a> {
    pub fn new(
        model: &'a mut EditorModel,
        display_string: &'a str,
        term_height: u16,
        term_width: u16,
    ) -> Self {
        Self {
            model: model,
            display_content: string_to_lines(display_string, term_width, 1, 0, false),
            first_line_num: 1,
            first_char_ind: 0,
            scroll_amount: 0,
            scroll_help_amount: 0,
            mode: Mode::Normal,
            show_line_nums: false,
            msg_display: vec![],
            search_term: None,
            cursor_pos: (1, 1),
            term_size: (term_height, term_width),
            running: true,
        }
    }

    /*
     * Get ___ methods below:
     */
    pub fn get_filename(&self) -> &str {
        return &self.model.file_name.as_str();
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
    pub fn get_app_mode(&self) -> &Mode {
        return &self.mode;
    }
    pub fn get_msg_display(&self) -> String {
        return self.msg_display.iter().collect();
    }
    pub fn get_cursor_pos(&self) -> (u16, u16) {
        return self.cursor_pos;
    }
    pub fn get_scroll_amount(&self) -> u16 {
        return self.scroll_amount;
    }
    pub fn get_scroll_help_amount(&self) -> u16 {
        return self.scroll_help_amount;
    }
    pub fn get_term_size(&self) -> (u16, u16) {
        return self.term_size;
    }
    pub fn get_search_term(&self) -> &Option<String> {
        return &self.search_term;
    }
    pub fn get_show_line_num(&self) -> bool {
        return self.show_line_nums;
    }

    /*
     * Used by View to show the current mode, and important inputs
     */
    pub fn get_mode_text(&self) -> &str {
        match &self.mode {
            Mode::Normal => return "Normal Mode [h]=>Help [i]=>Insert [:]=>Command [/]=>Search",
            Mode::Command => return "Command Mode [ENTER]=>Submit [ESC]=>Exit",
            Mode::SearchInput => return "Search Mode [ENTER]=>Submit [ESC]=>Exit",
            Mode::Search => return "Search Mode [n]=>Next [p]=>Prev [ESC]=>Exit",
            Mode::Insert => return "Insertion Mode [ESC]=>Exit",
            Mode::Minimized => return "Please Enlarge Terminal Window",
            Mode::Help => return "Help Page [ESC]=>Exit [^][v] to Scroll Help Text",
        }
    }

    /*
     * Used to index app's DisplayContent vector using cursor coordinates and scroll amount
     */
    pub fn get_cursor_display_row(&self) -> usize {
        // display_content is 0-indexed, cursor_pos is 1-indexed
        return (self.scroll_amount + self.cursor_pos.0) as usize - 1;
    }
    /*
     * Used to get which column of the file line the cursor is currently located at
     */
    pub fn get_cursor_inline_index(&self) -> usize {
        // Get the number of non-character columns in the current line
        let line = &self.display_content[self.get_cursor_display_row()];
        let invalid_cols = &line.invalid_cols;
        let num_skipped_cols = invalid_cols
            .iter()
            .filter(|col| col < &&self.cursor_pos.1)
            .count();

        // Sum the inline index of the displayed line the cursor is on, with the cursor position, and subtract non-char columns
        let mut index = &line.inline_index + (self.cursor_pos.1 as usize) - num_skipped_cols;
        if let Mode::Insert = self.mode {
            index -= 1; // Insertion mode has a thinner cursor that can move into 0 indexing
        }
        if self.show_line_nums {
            index -= (line.line_num.count_digits() as usize + 1); // subtract the line number characters
        }
        return index;
    }

    /*
     * Used to get the character index of the cursor in the entire file
     */
    pub fn get_cursor_file_index(&self) -> usize {
        let line = &self.display_content[self.get_cursor_display_row()];
        let invalid_cols = &line.invalid_cols;
        let num_skipped_cols = invalid_cols
            .iter()
            .filter(|col| col < &&self.cursor_pos.1)
            .count();
        if let Mode::Insert = self.mode {
            return &line.infile_index + (self.cursor_pos.1 as usize) - num_skipped_cols - 1;
        } else {
            return &line.infile_index + (self.cursor_pos.1 as usize) - num_skipped_cols;
        }
    }

    /*
     * Defines the boundaries of the cursor in the terminal window
     */
    pub fn term_top_cursor_bound(&self) -> u16 {
        return 1;
    }
    pub fn term_bottom_cursor_bound(&self) -> u16 {
        // -4 because borders + bottom status window
        return self.term_size.0 - 4;
    }
    pub fn term_left_cursor_bound(&self) -> u16 {
        if self.show_line_nums {
            return self.display_content[self.get_cursor_display_row()]
                .line_num
                .count_digits() as u16
                + 2;
        }
        return 1;
    }
    pub fn term_right_cursor_bound(&self) -> u16 {
        // - 2 because borders
        return self.term_size.1 - 2;
    }

    /*
     * Used to handle updates in the terminal size
     */
    pub fn update_term_size(&mut self, term_height: u16, term_width: u16) {
        self.term_size = (term_height, term_width);

        if term_height <= 4 {
            self.mode = Mode::Minimized;
            self.cursor_pos = (1, 1);
            return;
        }
        if let Mode::Minimized = self.mode {
            if term_height > 4 {
                self.mode = Mode::Normal;
            }
        }

        // TO DO: Future should use ref to buffer instead of display_string
        self.display_content = string_to_lines(
            self.model.rope.to_string().as_str(),
            term_width,
            self.first_line_num,
            self.first_char_ind,
            self.show_line_nums,
        );

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
            // TO DO: Attempt to load into buffer before giving up and shifting cursor back up
            self.cursor_pos.0 = self.display_content.len() as u16;
        }

        // Ensure the cursor stays in a valid location
        self.snap_cursor();
        self.slip_cursor();
    }

    /*
     * Used to start/stop the app
     */
    pub fn running(&self) -> bool {
        return self.running;
    }
    fn exit(&mut self) {
        self.running = false;
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
            Mode::Minimized => {}
            Mode::Help => self.help_handle_key_event(key_event),
        }
    }

    fn help_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.scroll_help_amount = 0;
            }
            KeyCode::Up | KeyCode::Char('^') => self.scroll_help_up(),
            KeyCode::Down | KeyCode::Char('v') | KeyCode::Char('V') => self.scroll_help_down(),
            _ => {}
        };
    }

    fn normal_handle_key_event(&mut self, key_event: KeyEvent) {
        // Clear any error/status messages once the user makes an input
        self.msg_display = vec![];
        match key_event.code {
            KeyCode::Char('i') | KeyCode::Char('I') => self.mode = Mode::Insert,
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.msg_display = vec![':'];
            }
            KeyCode::Char('/') => {
                self.mode = Mode::SearchInput;
                self.msg_display = vec!['/'];
            }
            KeyCode::Up | KeyCode::Char('k') => self.cursor_up(),
            KeyCode::Down | KeyCode::Char('j') => self.cursor_down(),
            KeyCode::Left | KeyCode::Char('h') => self.cursor_left(),
            KeyCode::Right | KeyCode::Char('l') => self.cursor_right(),
            _ => {}
        };
    }

    fn command_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.msg_display = vec![];
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                let command: String = self.msg_display.iter().collect();
                match command.as_str() {
                    ":w" | ":W" | ":write" => { /* TO DO */ }
                    ":q" | ":Q" | ":quit" => self.exit(),
                    ":wq" | ":WQ" => { /* TO DO */ }
                    ":set number" | ":set num" | ":set nu" | ":num" | ":nu" => {
                        self.show_line_nums = !self.show_line_nums;
                        // TO DO: Make sure to pass in string ref to buffer (smth like that) where self.display_string is below to update View
                        self.display_content = string_to_lines(
                            self.model.rope.to_string().as_str(),
                            self.term_size.1,
                            self.first_line_num,
                            self.first_char_ind,
                            self.show_line_nums,
                        );
                        self.slip_cursor(); // mainly used when turning on show_line_nums to stay out of line num region
                        self.snap_cursor(); // mainly used when turning off show_line_nums to snap to end of short lines
                        self.msg_display = vec![];
                    }
                    _ => {
                        let error_msg = String::from("Error: Invalid Command");
                        self.msg_display = error_msg.chars().collect();
                    }
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.msg_display.pop();
                if self.msg_display.len() == 0 {
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Char(character) => self.msg_display.push(character),
            _ => {}
        }
    }

    fn insert_handle_key_event(&mut self, key_event: KeyEvent) {
        // Clear any error/status messages once the user makes an input
        self.msg_display = vec![];
        match key_event.code {
            KeyCode::Esc => {
                self.msg_display = vec![];
                self.mode = Mode::Normal;
                self.snap_cursor();
            }
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Enter => self.insert_char('\n'),
            KeyCode::Tab => self.insert_char('\t'),
            KeyCode::Char(character) => self.insert_char(character),
            KeyCode::Up => self.cursor_up(),
            KeyCode::Down => self.cursor_down(),
            KeyCode::Left => self.cursor_left(),
            KeyCode::Right => self.cursor_right(),
            _ => {}
        }
    }
    fn delete_char(&mut self) {
        let file_ind = self.get_cursor_file_index(); // char index of file where character should be deleted
        self.model.delete_char(file_ind);
        self.display_content = string_to_lines(
            self.model.rope.to_string().as_str(),
            self.term_size.1,
            self.first_line_num,
            self.first_char_ind,
            self.show_line_nums,
        );
        self.cursor_left();
    }
    fn insert_char(&mut self, c: char) {
        let file_ind = self.get_cursor_file_index(); // char index of file where character should be inserted
        self.model.insert_char(c, file_ind);
        self.display_content = string_to_lines(
            self.model.rope.to_string().as_str(),
            self.term_size.1,
            self.first_line_num,
            self.first_char_ind,
            self.show_line_nums,
        );
        self.cursor_right();
    }

    fn search_input_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.msg_display = vec![];
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                // TO DO: Actually perform the search with buffer instead of dummy_search. Only transition to search mode if matches found
                if let Some(num_matches) = self.dummy_search() {
                    self.search_term = Some(self.msg_display[1..].iter().collect());
                    let mut message = num_matches.to_string();
                    message.push_str(" matches");
                    self.msg_display = message.chars().collect();
                    self.mode = Mode::Search;
                } else {
                    self.msg_display = "Error: No matches found".chars().collect();
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Backspace => {
                self.msg_display.pop();
                if self.msg_display.len() == 0 {
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Char(character) => self.msg_display.push(character),
            _ => {}
        }
    }
    fn dummy_search(&self) -> Option<usize> {
        let query: String = self.msg_display[1..].iter().collect();
        if self.model.rope.to_string().contains(&query) {
            return Some(self.model.rope.to_string().matches(&query).count());
        } else {
            return None;
        }
    }

    fn search_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.search_term = None;
                self.msg_display = vec![];
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') => { /*TO DO Scroll to line containing next match if it exists*/ }
            KeyCode::Char('p') => { /*TO DO Scroll to line containing previous match if it exists*/
            }
            _ => {}
        }
    }

    fn scroll_help_up(&mut self) {
        if self.scroll_help_amount > 0 {
            self.scroll_help_amount -= 1;
        }
    }
    fn scroll_help_down(&mut self) {
        self.scroll_help_amount = cmp::min(MAX_HELP_SCROLL, self.scroll_help_amount + 1);
    }
    fn scroll_up(&mut self) -> Result<(), &str> {
        if self.scroll_amount > 0 {
            self.scroll_amount -= 1;
            return Ok(());
        } else {
            // TO DO: Ask buffer to read in another line from rope to the head of buffer, and adjust scroll accordingly
            // Then, if the last line is no longer visible in the view,
            // push out a line from the end of buffer
            // Make sure to run self.display_content = string_to_lines with appropriate args to update
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
            // TO DO: Ask buffer to read in another line from rope to the end of buffer
            // Then, if self.first_line_num no longer matches the line num of the top visible line in the view,
            // push out a line from the head of buffer and adjust scroll accordingly.
            // Make sure to run self.display_content = string_to_lines with appropriate args to update
            return Err("Error: End of file reached");
        }
    }

    fn cursor_up(&mut self) {
        // Check if there is room to move the cursor upwards
        if self.cursor_pos.0 > self.term_top_cursor_bound() {
            // Move up, and adjust cursor to a viable position
            self.cursor_pos.0 -= 1;
            self.snap_cursor();
            self.slip_cursor();
        } else {
            // If at top bound, try to scroll content instead of moving cursor
            if let Err(msg) = self.scroll_up() {
                self.msg_display = msg.chars().collect();
            };
        }
    }

    fn cursor_down(&mut self) {
        // Edge case: small file, big terminal. End of file was reached
        if (self.cursor_pos.0 as usize) == self.display_content.len() {
            self.msg_display = "Error: End of file reached".chars().collect();
            return;
        }

        // Check if there is room to move cursor downwards
        if self.cursor_pos.0 < self.term_bottom_cursor_bound() {
            // Move down, and adjust cursor to a viable position
            self.cursor_pos.0 += 1;
            self.snap_cursor();
            self.slip_cursor();
        } else {
            // If at bottom bound, try to scroll content instead of moving cursor
            if let Err(msg) = self.scroll_down() {
                self.msg_display = msg.chars().collect();
            };
        }
    }

    fn cursor_right(&mut self) {
        let line = &self.display_content[self.get_cursor_display_row()];

        // If cursor will move into the middle of a wide character (ex tab space) 'slip' it rightwards until the next character is valid
        let invalid_cols = &line.invalid_cols;
        while invalid_cols.contains(&(self.cursor_pos.1 + 1)) {
            self.cursor_pos.1 += 1;
        }

        let mut bound = width(&line.line_content);
        // Allow the cursor to move to the end of the line if in insertion mode
        if let Mode::Insert = self.mode {
            bound += 1;
        }

        // If cursor is at or past the right boundary (end of the line), move to the start of the next line if available
        if self.cursor_pos.1 as u64 >= bound {
            // Edge case: small file, big terminal. End of file was reached
            if (self.get_cursor_display_row() + 1) == self.display_content.len() {
                self.msg_display = "Error: End of file reached".chars().collect();
                return;
            }

            // If scrolling needed, try to do so
            if self.cursor_pos.0 == self.term_bottom_cursor_bound() {
                if let Err(msg) = self.scroll_down() {
                    self.msg_display = msg.chars().collect();
                    return;
                };
            } else {
                // Only move down if did not scroll
                self.cursor_pos.0 += 1;
            }
            self.cursor_pos.1 = self.term_left_cursor_bound(); // Move to start of next line
        } else {
            self.cursor_pos.1 += 1; // Move cursor one step to the right
        }
    }

    fn cursor_left(&mut self) {
        // If the cursor is at the start of the line, move to the end of the previous line if available
        if self.cursor_pos.1 == self.term_left_cursor_bound() {
            // If scrolling needed, try to do so
            if self.cursor_pos.0 == self.term_top_cursor_bound() {
                if let Err(msg) = self.scroll_up() {
                    self.msg_display = msg.chars().collect();
                    return;
                };
            } else {
                // Only move up if did not scroll
                self.cursor_pos.0 -= 1;
            }
            // Get end of line coordinates
            let line = &self.display_content[self.get_cursor_display_row()].line_content;
            let mut bound = width(line);

            // Allow the cursor to move one space further if in insertion mode
            if let Mode::Insert = self.mode {
                bound += 1;
            }

            self.cursor_pos.1 = bound as u16; // Move to end of prev line
        } else {
            self.cursor_pos.1 -= 1; // Move cursor one step to the left
        }
        self.slip_cursor();
    }

    // Snap cursor to end of line after moving up/down into a shorter line of text
    fn snap_cursor(&mut self) {
        // Get end of line coordinates
        let line = &self.display_content[self.get_cursor_display_row()].line_content;
        let mut bound = width(line);

        // Allow the cursor to move to the end of the line if in insertion mode
        if let Mode::Insert = self.mode {
            bound += 1;
        }

        // Snap cursor to end of line after moving up
        let line_len = bound as u16;
        if line_len < self.cursor_pos.1 {
            self.cursor_pos.1 = line_len;
        }
    }

    // If cursor just moved into the middle of a wide character (ex tab space) 'slip' it leftwards to valid space
    // Also used to keep the cursor out of the line numbers
    fn slip_cursor(&mut self) {
        let invalid_cols = &self.display_content[self.get_cursor_display_row()].invalid_cols;
        while invalid_cols.contains(&self.cursor_pos.1) {
            self.cursor_pos.1 -= 1;
        }

        // Ensure cursor shifts out of the line number region
        while self.cursor_pos.1 < self.term_left_cursor_bound() {
            self.cursor_pos.1 += 1;
        }
    }
}
