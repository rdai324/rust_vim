use crate::App;
use count_digits::{self, CountDigits};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::cmp::max;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let file_name = app.get_filename();
    let file_line = app.get_fileline();
    let cursor_pos = app.get_cursor_pos();
    let display_content = app.get_content();
    let app_mode = app.get_mode();
    let ui_message = app.get_ui_display();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(2)])
        .split(frame.area());

    // Used for line numbers and file contents
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(0), Constraint::Min(1)])
        .split(layout[0]);

    // Used for cursor coordinates, command line, and error messages
    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            // usize to u16 conversion should be safe, since the number of digits in the cursor should be small
            Constraint::Length(
                5 + max(
                    (file_line + (cursor_pos.0 as usize) - 1).count_digits(),
                    cursor_pos.1.count_digits(),
                ) as u16,
            ),
            Constraint::Min(12),
        ])
        .split(layout[1]);

    // File Content
    let title = Line::from(file_name.bold());
    let content_block = Block::bordered().title(title).border_set(border::THICK);

    // Split display contents string into lines based on unicode char width, and newline characters

    let content = Paragraph::new(display_content).block(content_block);
    frame.render_widget(content, main_layout[1]);

    // Cursor Location
    let cursor_row_text = format!("row: {}", cursor_pos.0);
    let cursor_col_text = format!("col: {}", cursor_pos.1);
    let cursor_pos_content: Text = vec![cursor_row_text.into(), cursor_col_text.into()].into();
    frame.render_widget(Paragraph::new(cursor_pos_content), bottom_layout[0]);

    // Command Window
    let mode_text = Line::styled(
        app_mode,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .centered();
    let ui_text = Line::styled(ui_message, Style::default().fg(Color::White)).centered();
    let ui_content: Text = vec![mode_text.into(), ui_text.into()].into();
    let ui_block = Block::new().borders(Borders::LEFT);
    frame.render_widget(Paragraph::new(ui_content).block(ui_block), bottom_layout[1]);

    // Render cursor
    frame.set_cursor_position(Position::new(cursor_pos.1, cursor_pos.0));
}
