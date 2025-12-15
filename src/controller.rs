use crate::model::EditorModel;
use crate::view::MAX_HELP_SCROLL;
use core::ops::Range;
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
    Insert,
    Minimized, //Used to prevent cursor out of bounds crash when terminal is shrunk to <=4 lines tall
    Help,      // Used to display the help screen
    Quit,      // Used for :q quit popup
}

#[derive(Debug)]
pub enum QuitSelection {
    SaveAndQuit,
    NoSaveQuit,
    Cancel,
}

fn string_to_lines(
    string: &str,
    term_width: u16,
    show_lines: bool,
    highlight_ranges: &Vec<Range<usize>>,
) -> Vec<DisplayLine> {
    let chars = string.chars();
    let max_line_len = term_width - 2; // to accomodate the two borders
    let mut highlight_ranges = highlight_ranges.iter(); // typecast to iterator for easier use
    let mut highlight_range = highlight_ranges.next();
    let mut highlight_start_idx: Option<usize> = None;

    // Start constructing vector of wrapped display lines
    let mut lines: Vec<DisplayLine> = Vec::new();
    let mut line = DisplayLine::new(1, 0, 0); // First (1) line has infile and inline char indices of 0
    let mut num_chars = 0; // Used for calculating file indexing
    let mut total_char_width = 0; // Used to track how many characters can fit on a line

    // Add line numbers to first line
    if show_lines {
        line.line_content = String::from("1|");
        total_char_width = 2;
    }

    // Start wrapping lines
    for character in chars {
        // if character is a newline, stop building line and append it to lines
        if character == '\n' {
            // First check if we are still highlighting
            if let Some(start) = highlight_start_idx {
                if show_lines {
                    line.highlight_ranges.push(Range {
                        start: start + line.line_num.count_digits() + 1,
                        end: num_chars + line.line_num.count_digits() + 1,
                    })
                } else {
                    line.highlight_ranges.push(Range {
                        start: start,
                        end: num_chars,
                    });
                }

                if highlight_range.unwrap().end > line.infile_index + num_chars {
                    highlight_start_idx = Some(0);
                } else {
                    highlight_start_idx = None;
                    highlight_range = highlight_ranges.next();
                }
            }

            // Add current line and build next one
            lines.push(line);
            let last_line = &lines[lines.len() - 1];
            line = DisplayLine::new(
                last_line.line_num + 1,
                last_line.infile_index + num_chars + 1,
                0,
            );
            num_chars = 0;
            total_char_width = 0;

            // Line Numbers for new line
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

            // First check if we are still highlighting
            if let Some(start) = highlight_start_idx {
                if show_lines {
                    line.highlight_ranges.push(Range {
                        start: start + line.line_num.count_digits() + 1,
                        end: num_chars + line.line_num.count_digits() + 1,
                    })
                } else {
                    line.highlight_ranges.push(Range {
                        start: start,
                        end: num_chars,
                    });
                }
                highlight_start_idx = Some(0);
            }

            // Add current line and build next one
            lines.push(line);
            let last_line = &lines[lines.len() - 1];
            line = DisplayLine::new(
                last_line.line_num,
                last_line.infile_index + num_chars,
                last_line.inline_index + num_chars,
            );
            num_chars = 0;
            total_char_width = 0;

            // Line Numbers for new line
            if show_lines {
                line.line_content = line.line_num.to_string();
                line.line_content.push('|');
                total_char_width = line.line_num.count_digits() as u16 + 1;
            }
        }

        // Add character to line to be rendered
        // Handle tabs specially due to dynamic sizes
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
            // Render character and...
            line.line_content.push(character);
            num_chars += 1;
            // ...Keep track of 'invalid columns' for cursor due to wide characters
            for i in 1..char_width {
                line.invalid_cols.push(total_char_width + 1 + i)
            }
            total_char_width += char_width;
        }

        // Check Highlighting

        // Check if the char added is the last char of a match
        let infile_char_idx = line.infile_index + num_chars - 1;
        if let Some(start) = highlight_start_idx {
            if highlight_range.unwrap().end == infile_char_idx {
                if show_lines {
                    line.highlight_ranges.push(Range {
                        start: start + line.line_num.count_digits() + 1,
                        end: num_chars + line.line_num.count_digits(),
                    })
                } else {
                    line.highlight_ranges.push(Range {
                        start: start,
                        end: num_chars - 1,
                    });
                }
                highlight_range = highlight_ranges.next();
                highlight_start_idx = None;
            }
        }

        // Check if the char added is the first char of a match
        if let Some(range) = highlight_range {
            if num_chars > 0 {
                if range.start == infile_char_idx {
                    highlight_start_idx = Some(num_chars - 1)
                }
            }
        }
    }

    // Push the last line to lines
    lines.push(line);
    return lines;
}

#[derive(Debug)]
pub struct DisplayLine {
    pub line_content: String,                // String to display in the terminal
    pub line_num: usize,                     // In file line number
    pub infile_index: usize, // Char index of the start of this displayed line in the total file
    pub inline_index: usize, // Char index of the start of this displayed line in the file line
    pub invalid_cols: Vec<u16>, // used to ensure cursor is never in the middle of a multi-column character
    pub highlight_ranges: Vec<Range<usize>>, // used to find which chars should be highlighted for search matching
}

impl DisplayLine {
    pub fn new(line_num: usize, infile_index: usize, inline_index: usize) -> Self {
        Self {
            line_content: String::from(""),
            line_num,
            infile_index,
            inline_index,
            invalid_cols: vec![],
            highlight_ranges: vec![],
        }
    }
}

#[derive(Debug)]
pub struct App<'a> {
    model: &'a mut EditorModel,
    display_content: Vec<DisplayLine>, // Vector of DisplayLine structs representing content being displayed + useful info
    scroll_amount: u16,                // How far did we scroll down display_content?
    scroll_help_amount: u16,           // How far to scroll help popup
    quit_selection: QuitSelection,
    mode: Mode,
    show_line_nums: bool,
    msg_display: Vec<char>, // What to show in the bottom message bar (user input, error messages, etc.)
    search_term: String, // What is being searched for in search mode. Only assigned on successful match for View highlighting
    match_ranges: Vec<Range<usize>>, // Infile char indexes of search matches found
    cursor_pos: (u16, u16), // cursor position in terminal. (y, x), or (row, col), with 1,1 being the top-left corner (1 not 0 due to border)
    term_size: (u16, u16),  // Terminal size (Num rows, num cols)
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
            display_content: string_to_lines(display_string, term_width, false, &vec![]),
            scroll_amount: 0,
            scroll_help_amount: 0,
            quit_selection: QuitSelection::Cancel,
            mode: Mode::Normal,
            show_line_nums: false,
            msg_display: vec![],
            search_term: String::new(),
            match_ranges: vec![],
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
    pub fn get_quit_selection(&self) -> &QuitSelection {
        return &self.quit_selection;
    }
    pub fn get_show_line_num(&self) -> bool {
        return self.show_line_nums;
    }

    // Are we highlighting search matches?
    pub fn get_show_highlights(&self) -> bool {
        return self.search_term.len() > 0;
    }

    /*
     * Used by View to show the current mode, and important inputs
     */
    pub fn get_mode_text(&self) -> &str {
        match &self.mode {
            Mode::Normal => return "Normal Mode [z]=>Help [i]=>Insert [:]=>Command [/]=>Search",
            Mode::Command => return "Command Mode [ENTER]=>Submit [ESC]=>Exit",
            Mode::SearchInput => return "Search Mode [ENTER]=>Submit [ESC]=>Exit",
            Mode::Insert => return "Insertion Mode [ESC]=>Exit",
            Mode::Minimized => return "Please Enlarge Terminal Window",
            Mode::Help => return "Help Page [ESC]=>Exit [^][v] to Scroll Help Text",
            Mode::Quit => return "[<][>] to select, [ENTER]=>Confirm, [ESC]=>Cancel",
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
            index -= line.line_num.count_digits() as usize + 1; // subtract the line number characters
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

        // Sum the infile index of the displayed line the cursor is on, with the cursor position, and subtract non-char columns
        let mut index = &line.infile_index + (self.cursor_pos.1 as usize) - num_skipped_cols;
        if let Mode::Insert = self.mode {
            index -= 1; // Insertion mode has a thinner cursor that can move into 0 indexing
        }
        if self.show_line_nums {
            index -= line.line_num.count_digits() as usize + 1; // subtract the line number characters
        }
        return index;
    }

    // Used to re-wrap the displayed text after it's been updated
    fn wrap_text(&mut self) {
        // If highlighting, rerun search to update highlighting
        if self.get_show_highlights() {
            self.match_ranges = self.model.run_search(self.search_term.as_str());
        }
        self.display_content = string_to_lines(
            self.model.rope.to_string().as_str(),
            self.term_size.1,
            self.show_line_nums,
            &self.match_ranges,
        );
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

        // Used to prevent panic from shrinking terminal too small
        if term_height <= 4 {
            self.mode = Mode::Minimized;
            self.cursor_pos = (1, 1);
            return;
        }
        // Use to return to normal functionality after enlarging terminal back to usable size
        if let Mode::Minimized = self.mode {
            if term_height > 4 {
                self.mode = Mode::Normal;
                // Clear any previous unsubmitted user input
                if !self.get_show_highlights() {
                    self.msg_display = vec![];
                } else {
                    // Re-display search matches message if we are still highlighting
                    self.msg_display = format!(
                        "{} matches for {}",
                        self.match_ranges.len().to_string(),
                        &self.search_term
                    )
                    .chars()
                    .collect();
                }
            }
        }

        // Re-wrap display content
        self.wrap_text();

        // Update cursor position if terminal size shrunk
        if self.cursor_pos.1 > self.term_right_cursor_bound() {
            self.cursor_pos.1 = self.term_right_cursor_bound();
        }
        if self.cursor_pos.0 > self.term_bottom_cursor_bound() {
            self.cursor_pos.0 = self.term_bottom_cursor_bound();
        }

        // Edge case if enlarging terminal window and unwrapping displayed text reduced number of rows occupied by text
        if self.cursor_pos.0 > self.display_content.len() as u16 {
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
            // Handle terminal resizing
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
            Mode::Minimized => {}
            Mode::Help => self.help_handle_key_event(key_event),
            Mode::Quit => self.quit_handle_key_event(key_event),
        }
    }

    fn quit_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Change selected option
            KeyCode::Left | KeyCode::Char('<') | KeyCode::Char(',') => match self.quit_selection {
                QuitSelection::Cancel => self.quit_selection = QuitSelection::NoSaveQuit,
                QuitSelection::NoSaveQuit => self.quit_selection = QuitSelection::SaveAndQuit,
                QuitSelection::SaveAndQuit => {}
            },
            KeyCode::Right | KeyCode::Char('>') | KeyCode::Char('.') => match self.quit_selection {
                QuitSelection::Cancel => {}
                QuitSelection::NoSaveQuit => self.quit_selection = QuitSelection::Cancel,
                QuitSelection::SaveAndQuit => self.quit_selection = QuitSelection::NoSaveQuit,
            },
            // Confirm selection
            KeyCode::Enter => match self.quit_selection {
                // Cancel and return to normal mode
                QuitSelection::Cancel => {
                    self.mode = Mode::Normal;
                    if self.get_show_highlights() {
                        // Re-display search matches message if we are still highlighting
                        self.msg_display = format!(
                            "{} matches for {}",
                            self.match_ranges.len().to_string(),
                            &self.search_term
                        )
                        .chars()
                        .collect();
                    } else {
                        // Clear quit command
                        self.msg_display = vec![];
                    }
                }
                QuitSelection::NoSaveQuit => self.exit(),
                QuitSelection::SaveAndQuit => match self.model.save() {
                    Ok(_) => {
                        self.exit();
                    }
                    Err(e) => {
                        self.msg_display = format!("Error: could not write file: {}", e)
                            .chars()
                            .collect();
                    }
                },
            },
            // Cancel and return to Normal Mode
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                if self.get_show_highlights() {
                    // Re-display search matches message if we are still highlighting
                    self.msg_display = format!(
                        "{} matches for {}",
                        self.match_ranges.len().to_string(),
                        &self.search_term
                    )
                    .chars()
                    .collect();
                } else {
                    // Clear quit command
                    self.msg_display = vec![];
                }
            }
            _ => {}
        }
    }

    fn help_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Close help pop-up
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.scroll_help_amount = 0;

                // Re-display search matches message if we are still highlighting
                if self.get_show_highlights() {
                    self.msg_display = format!(
                        "{} matches for {}",
                        self.match_ranges.len().to_string(),
                        &self.search_term
                    )
                    .chars()
                    .collect();
                }
            }
            // Scroll help pop-up contents
            KeyCode::Up | KeyCode::Char('^') => self.scroll_help_up(),
            KeyCode::Down | KeyCode::Char('v') | KeyCode::Char('V') => self.scroll_help_down(),
            _ => {}
        };
    }

    fn normal_handle_key_event(&mut self, key_event: KeyEvent) {
        // Clear any error/status messages once the user makes an input
        if !self.get_show_highlights() {
            self.msg_display = vec![];
        } else {
            // Re-display search matches message if we are still highlighting
            self.msg_display = format!(
                "{} matches for {}",
                self.match_ranges.len().to_string(),
                &self.search_term
            )
            .chars()
            .collect();
        }
        match key_event.code {
            // Turn off any search highlighting
            KeyCode::Esc => {
                self.search_term = String::new();
                self.match_ranges = vec![];
                self.msg_display = vec![];
            }
            // Enter insert mode
            KeyCode::Char('i') | KeyCode::Char('I') => self.mode = Mode::Insert,
            // Enter command mode
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.msg_display = vec![':'];
            }
            // Enter search mode
            KeyCode::Char('/') => {
                self.mode = Mode::SearchInput;
                self.search_term = String::new();
                self.match_ranges = vec![];
                self.msg_display = vec!['/'];
            }
            // Open help popup
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                self.mode = Mode::Help;
                self.msg_display = vec![];
            }
            // Move cursor
            KeyCode::Up | KeyCode::Char('k') => self.cursor_up(),
            KeyCode::Down | KeyCode::Char('j') => self.cursor_down(),
            KeyCode::Left | KeyCode::Char('h') => self.cursor_left(),
            KeyCode::Right | KeyCode::Char('l') => self.cursor_right(),
            _ => {}
        };
    }

    fn command_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Cancel and exit to Normal Mode
            KeyCode::Esc => {
                if self.get_show_highlights() {
                    // Re-display search matches message if we are still highlighting
                    self.msg_display = format!(
                        "{} matches for {}",
                        self.match_ranges.len().to_string(),
                        &self.search_term
                    )
                    .chars()
                    .collect();
                } else {
                    // Clear any remaining user input
                    self.msg_display = vec![];
                }
                self.mode = Mode::Normal;
            }
            // Submit command and execute if it exists
            KeyCode::Enter => {
                let command: String = self.msg_display.iter().collect();
                match command.as_str() {
                    // Write to file
                    ":w" | ":write" => {
                        match self.model.save() {
                            Ok(_) => {
                                self.msg_display = "Wrote file".chars().collect();
                            }
                            Err(e) => {
                                self.msg_display = format!("Error: could not write file: {}", e)
                                    .chars()
                                    .collect();
                            }
                        }
                        self.mode = Mode::Normal;
                    }
                    // Attempt to Quit without saving
                    ":q" | ":quit" => self.mode = Mode::Quit,
                    // Write and quit
                    ":wq" => match self.model.save() {
                        Ok(_) => {
                            self.exit();
                        }
                        Err(e) => {
                            self.msg_display = format!("Error: could not write file: {}", e)
                                .chars()
                                .collect();

                            self.mode = Mode::Normal;
                        }
                    },
                    // Toggle line numbers
                    ":set number" | ":set num" | ":set nu" | ":num" | ":nu" => {
                        self.show_line_nums = !self.show_line_nums;
                        // Re-wrap display content for view
                        self.wrap_text();
                        self.snap_cursor(); // mainly used when turning off show_line_nums to snap to end of short lines
                        self.slip_cursor(); // mainly used when turning on show_line_nums to stay out of line num region

                        if self.get_show_highlights() {
                            // Re-display search matches message if we are still highlighting
                            self.msg_display = format!(
                                "{} matches for {}",
                                self.match_ranges.len().to_string(),
                                &self.search_term
                            )
                            .chars()
                            .collect();
                        } else {
                            // clear the command from the window
                            self.msg_display = vec![];
                        }
                        self.mode = Mode::Normal;
                    }
                    // Delete current file line at cursor
                    ":dd" => {
                        let deleting_line_num =
                            self.display_content[self.get_cursor_display_row()].line_num;

                        let mut start_idx = 0;
                        let mut start_found = false;
                        let mut end_found = false;
                        let mut curr_display_row = 0;
                        // Search for the start of the line being deleted, and the start of the line after it
                        while curr_display_row < self.display_content.len() {
                            let line = &self.display_content[curr_display_row];

                            if start_found {
                                if line.line_num != deleting_line_num {
                                    let end_idx = line.infile_index;
                                    self.model.delete_range(start_idx, end_idx);
                                    end_found = true;
                                    break;
                                }
                            } else {
                                if line.line_num == deleting_line_num {
                                    start_idx = line.infile_index;
                                    start_found = true;
                                }
                            }

                            curr_display_row += 1;
                        }

                        // If we reached the end of the file before finding a new line, then this is the last line.
                        // Delete to the end
                        if !end_found {
                            self.model.delete_to_end(start_idx);
                        }

                        // Re-wrap displayed text
                        self.wrap_text();

                        // Move cursor upwards if :dd ended up leaving cursor out of bounds
                        while self.get_cursor_display_row() >= self.display_content.len() {
                            self.cursor_pos.0 -= 1;
                        }
                        self.snap_cursor();
                        self.slip_cursor();

                        // Return to normal mode and clear :dd command from message display
                        self.mode = Mode::Normal;
                        self.msg_display = vec![];
                    }
                    // Invalid command
                    _ => {
                        let error_msg = String::from("Error: Invalid Command");
                        self.mode = Mode::Normal;
                        self.msg_display = error_msg.chars().collect();
                    }
                }
            }
            // Delete right-most user input character
            KeyCode::Backspace => {
                self.msg_display.pop();
                if self.msg_display.len() == 0 {
                    if self.get_show_highlights() {
                        // Re-display search matches message if we are still highlighting
                        self.msg_display = format!(
                            "{} matches for {}",
                            self.match_ranges.len().to_string(),
                            &self.search_term
                        )
                        .chars()
                        .collect();
                    }
                    self.mode = Mode::Normal;
                }
            }
            // Type into user input
            KeyCode::Char(character) => self.msg_display.push(character),
            _ => {}
        }
    }

    fn insert_handle_key_event(&mut self, key_event: KeyEvent) {
        // Clear any error/status messages once the user makes an input
        if self.get_show_highlights() {
            // Re-display search matches message if we are still highlighting
            self.msg_display = format!(
                "{} matches for {}",
                self.match_ranges.len().to_string(),
                &self.search_term
            )
            .chars()
            .collect();
        } else {
            self.msg_display = vec![];
        }
        match key_event.code {
            // Exit to normal mode
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.snap_cursor();
                self.slip_cursor();
            }
            // Delete characters
            KeyCode::Backspace => {
                if self.get_cursor_file_index() > 0 {
                    self.cursor_left();
                    self.delete_char();
                } else {
                    self.msg_display = "Error: Start of file reached".chars().collect();
                }
            }
            KeyCode::Delete => self.delete_char(),
            // Type characters
            KeyCode::Enter => self.insert_char('\n'),
            KeyCode::Tab => self.insert_char('\t'),
            KeyCode::Char(character) => self.insert_char(character),
            // Move cursor
            KeyCode::Up => self.cursor_up(),
            KeyCode::Down => self.cursor_down(),
            KeyCode::Left => self.cursor_left(),
            KeyCode::Right => self.cursor_right(),
            _ => {}
        }
    }
    fn delete_char(&mut self) {
        let file_ind = self.get_cursor_file_index(); // char index of file where character should be deleted
        if self.model.delete_char(file_ind) {
            // Re-wrap file content for display
            self.wrap_text();
        } else {
            self.msg_display = "Error: End of file reached".chars().collect();
        }
    }
    fn insert_char(&mut self, c: char) {
        let file_ind = self.get_cursor_file_index(); // char index of file where character should be inserted
        self.model.insert_char(c, file_ind);

        // Re-wrap file content for display
        self.wrap_text();
        self.cursor_right();

        // If inserted at end of display line, need to shift cursor right one more time to be right of the new character on the new line
        if self.cursor_pos.1 == 1 && c != '\n' {
            self.cursor_right();
        }
    }

    fn search_input_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Cancel and exit to Normal Mode
            KeyCode::Esc => {
                self.msg_display = vec![];
                self.mode = Mode::Normal;
            }
            // Submit search query and see if matches are found
            KeyCode::Enter => {
                let search_query: String = self.msg_display[1..].iter().collect();
                self.match_ranges = self.model.run_search(search_query.as_str());
                if self.match_ranges.len() > 0 {
                    // Matches found, update app state so view knows to highlight them
                    self.wrap_text();
                    self.search_term = search_query;
                    self.msg_display = format!(
                        "{} matches for {}",
                        self.match_ranges.len().to_string(),
                        &self.search_term
                    )
                    .chars()
                    .collect();
                } else {
                    self.msg_display = "Error: No matches found".chars().collect();
                }
                self.mode = Mode::Normal;
            }
            // Delete right-most user input character
            KeyCode::Backspace => {
                self.msg_display.pop();
                // If entire user input deleted, return to normal mode
                if self.msg_display.len() == 0 {
                    self.mode = Mode::Normal;
                }
            }
            // Type into user input
            KeyCode::Char(character) => self.msg_display.push(character),
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
    // Scroll content up, but do not let it scroll out of bounds
    fn scroll_up(&mut self) -> Result<(), &str> {
        if self.scroll_amount > 0 {
            self.scroll_amount -= 1;
            return Ok(());
        } else {
            return Err("Error: Start of file reached");
        }
    }
    // Scroll content down, but do not let it scroll out of bounds
    fn scroll_down(&mut self) -> Result<(), &str> {
        let max_scroll_amount = self.display_content.len()
            - (self.term_bottom_cursor_bound() - self.term_top_cursor_bound() + 1) as usize;
        if (self.scroll_amount as usize) < max_scroll_amount {
            self.scroll_amount += 1;
            return Ok(());
        } else {
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
        let mut bound = cmp::max(width(line), 1);

        // Allow the cursor to move to the end of the line if in insertion mode
        if let Mode::Insert = self.mode
            && line.len() > 0
        {
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
