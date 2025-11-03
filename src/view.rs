use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::io;

pub enum Mode {
    Normal,
    Command,
    Search,
    Insert,
}

#[derive(Debug, Default)]
pub struct View {
    //filename: &'a str,
    //display_content: &'a str,
    //mode: Mode
    cursor_pos_x: usize,
    cursor_pos_y: usize,
    running: bool,
}

impl View {
    pub fn new() -> Self {
        Self {
            cursor_pos_x: 0,
            cursor_pos_y: 0,
            running: true,
        }
    }

    /*
     * "main loop" of the program which renders
     */
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame));
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
    }

    /*
     * Accepts any user inputs provided via crossterm while the program is running,
     * and passes them to the Controller for further processing
     */
    fn handle_events(&mut self) -> io::Result<()> {
        // TO DO: event::read is a blocking call, consider using event::poll instead?
        match event::read()? {
            // Check that this was a key press event
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    self.handle_key_event(key_event)
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(), // Temp exit command
            KeyCode::Up => { /* TO DO: Move the caret/cursor. */ }
            KeyCode::Down => { /* TO DO: Move the caret/cursor. */ }
            KeyCode::Left => { /* TO DO: Move the caret/cursor. */ }
            KeyCode::Right => { /* TO DO: Move the caret/cursor. */ }
            _ => { /* TO DO: Send to controller to process. */ }
        };
    }
}

impl Widget for &View {
    /*
     * Render's the View's UI
     */
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("File Name".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        Paragraph::new("").centered().block(block).render(area, buf);
    }
}
