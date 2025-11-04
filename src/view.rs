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
    filename: &'a str,        // Name of the file opened
    display_content: &'a str, // str slice representing the section of text currently being displayed in the View UI
    mode: Mode,
    caret_pos_x: u16,
    caret_pos_y: u16,
    running: bool,
}

impl<'a> View<'a> {
    pub fn new(filename: &'a mut str, display_content: &'a mut str) -> Self {
        Self {
            filename: filename,
            display_content: display_content,
            mode: Mode::Normal,
            caret_pos_x: 1,
            caret_pos_y: 1,
            running: true,
        }
    }

    /*
     * "main loop" which renders UI and listens for user input
     */
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        return Ok(());
    }

    fn exit(&mut self) {
        self.running = false;
    }

    fn draw(&self, frame: &mut Frame) {
        // Renders the View's UI by using its implementation of Widget::render() defined below
        frame.render_widget(self, frame.area());
        frame.set_cursor_position(Position::new(self.caret_pos_x, self.caret_pos_y));
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
            KeyCode::Up => self.caret_pos_y = cmp::max(self.caret_pos_y - 1, 1), // TO DO: Auto-snap cursor to end of line if needed.
            KeyCode::Down => self.caret_pos_y = self.caret_pos_y + 1, // TO DO: Auto-snap cursor to end of line if needed. Scroll when at the bottom
            KeyCode::Left => self.caret_pos_x = cmp::max(self.caret_pos_x - 1, 1),
            KeyCode::Right => self.caret_pos_x = self.caret_pos_x + 1, // TO DO: Limit how far right the caret can travel
            _ => { /* TO DO: Send to controller to process. */ }
        };
    }
}

impl<'a> Widget for &View<'a> {
    /*
     * Render's the View's UI
     *
     * TO DO:
     * - Render caret/cursor
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
            .wrap(Wrap { trim: false })
            .render(area, buf);

        /*
        io::stdout()
            .execute(MoveTo(self.caret_pos_x as u16, self.caret_pos_y as u16))?
            .execute(Show)?
            .execute(EnableBlinking);
        */
    }
}
