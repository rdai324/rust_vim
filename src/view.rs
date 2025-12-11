use crate::App;
use count_digits::{self, CountDigits};
use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::cmp::max;
use std::io::stdout;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let scroll_amount = app.get_scroll_amount();
    let cursor_pos = app.get_cursor_pos();

    let file_name = app.get_filename();
    let display_lines = app.get_content();
    let app_mode = app.get_mode();
    let ui_message = app.get_ui_display();
    let curr_row = display_lines[(scroll_amount + cursor_pos.0) as usize - 1].line_num;
    let curr_col = app.get_cursor_inline_index();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(2)])
        .split(frame.area());

    // Used for cursor coordinates, command line, and error messages
    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            // usize to u16 conversion and vice versa should be safe, since the number of digits in the cursor should be small
            Constraint::Length(5 + max(curr_row.count_digits(), curr_col.count_digits()) as u16),
            Constraint::Min(12),
        ])
        .split(layout[1]);

    // File Content
    let title = Line::from(file_name.bold());
    let content_block = Block::bordered().title(title).border_set(border::THICK);
    let mut display_content: Vec<Line> = Vec::new();
    for line in display_lines.iter() {
        display_content.push(Line::styled(&line.line_content, Style::default()));
    }
    let display_content: Text = display_content.into();

    let content = Paragraph::new(display_content)
        .block(content_block)
        .scroll((scroll_amount, 0));
    frame.render_widget(content, layout[0]);

    // Cursor Location
    let cursor_row_text = format!("row: {}", curr_row);
    let cursor_col_text = format!("col: {}", curr_col);
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
    let ui_text: Line;
    if ui_message.contains("Error") {
        ui_text =
            Line::styled(ui_message, Style::default().fg(Color::White).bg(Color::Red)).centered();
    } else {
        ui_text = Line::styled(ui_message, Style::default().fg(Color::White)).centered();
    }
    let ui_content: Text = vec![mode_text, ui_text].into();
    let ui_block = Block::new().borders(Borders::LEFT);
    frame.render_widget(Paragraph::new(ui_content).block(ui_block), bottom_layout[1]);

    // Render cursor
    if app_mode.contains("Insertion") {
        execute!(stdout(), SetCursorStyle::BlinkingBar).unwrap();
    } else {
        execute!(stdout(), SetCursorStyle::BlinkingBlock).unwrap();
    }
    frame.set_cursor_position(Position::new(cursor_pos.1, cursor_pos.0));
}
