use crossterm::{
    ExecutableCommand,
    cursor::{EnableBlinking, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Position, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use std::cmp;
use std::io;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Command,
    Search,
    Insert,
}

#[derive(Debug)]
pub struct View<'a> {
    filename: &'a str,           // Name of the file opened
    display_content: &'a str, // str slice representing the section of text currently being displayed in the View UI
    display_lines: Vec<&'a str>, //same as above, but split into lines for ease of use
    mode: Mode,               // current mode of the controller
    cursor_pos_x: u16, // Position of the cursor in the terminal. Note: Should stay greater than 1 due to border taking 1 character of space
    cursor_pos_y: u16,
    term_x: u16, // Size of the terminal window in characters
    term_y: u16,
    content_pos_x: usize, // Position of the cursor in the file. 0-indexed.
    content_pos_y: usize,
    scroll_x: u16, // Amount to scroll terminal contents
    scroll_y: u16,
    running: bool,
}

impl<'a> View<'a> {
    pub fn new(filename: &'a mut str, display_content: &'a mut str) -> Self {
        Self {
            filename: filename,
            display_content: display_content,
            display_lines: display_content.lines().collect(),
            mode: Mode::Normal,
            cursor_pos_x: 1,
            cursor_pos_y: 1,
            content_pos_x: 0,
            content_pos_y: 0,
            term_x: 0,
            term_y: 0,
            scroll_x: 0,
            scroll_y: 0,
            running: true,
        }
    }

    /*
     * "main loop" which renders UI and listens for user input
     */
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.term_x = terminal.size()?.width;
            self.term_y = terminal.size()?.height;
            self.handle_events()?;
        }
        return Ok(());
    }

    fn exit(&mut self) {
        self.running = false;
    }

    fn draw(&self, frame: &mut Frame) {
        // Renders the View's UI by using its implementation of Widget::render() defined below Terminal<CrosstermBackend<Stdout>>
        frame.render_widget(self, frame.area());
        frame.set_cursor_position(Position::new(self.cursor_pos_x, self.cursor_pos_y));
    }

    /*
     * Accepts any user inputs provided via crossterm while the program is running,
     * and passes them to the Controller for further processing
     */
    fn handle_events(&mut self) -> io::Result<()> {
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
            KeyCode::Up => self.step_cursor_up(), // TO DO: Auto-snap cursor to end of line if needed.
            KeyCode::Down => self.step_cursor_down(), // TO DO: Auto-snap cursor to end of line if needed. Scroll when at the bottom
            KeyCode::Left => self.cursor_pos_x = cmp::max(self.cursor_pos_x - 1, 1),
            KeyCode::Right => self.cursor_pos_x = self.cursor_pos_x + 1, // TO DO: Limit how far right the cursor can travel
            _ => { /* TO DO: Send to controller to process. */ }
        };
    }

    fn step_cursor_up(&mut self) {
        if self.cursor_pos_y == 1 {
            if self.content_pos_y > 0 {
                // At terminal boundary but more content exists, so scroll without moving cursor
                self.scroll_y = self.scroll_y - 1;
            }
        } else {
            // Move cursor
            self.cursor_pos_y = self.cursor_pos_y - 1;
        }
    }

    fn step_cursor_down(&mut self) {
        if self.cursor_pos_y == self.term_y - 2 {
            if self.content_pos_y < self.display_lines.len() - 1 {
                // At terminal boundary but more content exists, so scroll without moving cursor
                self.scroll_y = self.scroll_y + 1;
            }
        } else {
            // Move cursor
            self.cursor_pos_y = self.cursor_pos_y + 1;
        }
    }
}

impl<'a> Widget for &View<'a> {
    /*
     * Render's the View's UI
     *
     * TO DO:
     * - Render cursor/cursor
     * - Add scroll bar
     * - Add status/message bar at bottom for errors, command and search UI and details such as line #
     * -
     */
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(self.filename.bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        Paragraph::new(self.display_content)
            .block(block)
            .scroll((self.scroll_y, self.scroll_x))
            .render(area, buf);

        /*
        io::stdout()
            .execute(MoveTo(self.cursor_pos_x as u16, self.cursor_pos_y as u16))?
            .execute(Show)?
            .execute(EnableBlinking);
        */
    }
}
