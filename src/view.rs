use crate::App;
use count_digits::{self, CountDigits};
use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::cmp::max;
use std::io::stdout;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let file_name = app.get_filename();
    let display_lines = app.get_content();
    let show_line_num = app.get_show_line_num();
    let search_term = app.get_search_term();

    let app_mode = app.get_mode_text();
    let ui_message = app.get_msg_display();

    let scroll_amount = app.get_scroll_amount();
    let cursor_pos = app.get_cursor_pos();
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
    for line in display_lines
        .iter()
        .map(|display_line| &display_line.line_content)
    {
        let mut display_line = vec![];

        let mut line_content_index = 0;
        if show_line_num {
            line_content_index = line.find('|').unwrap();
            display_line.push(Span::styled(
                &line[..line_content_index],
                Style::default().fg(Color::Yellow),
            ));
        }

        if let Some(keyword) = search_term
            && line[line_content_index..].contains(keyword)
        {
            // There was a positive search result, highlight possible matches
            let mut substrings = line[line_content_index..].split(keyword);
            display_line.push(Span::raw(substrings.next().unwrap())); // The first elem of this iterator shouldn't be empty
            for substring in substrings {
                display_line.push(Span::styled(
                    keyword,
                    Style::default().fg(Color::White).bg(Color::Cyan),
                ));
                display_line.push(Span::raw(substring));
            }
        } else {
            display_line.push(Span::styled(&line[line_content_index..], Style::default()));
        }
        display_content.push(display_line.into());
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
    // First line describes mode and important keys
    let mode_text = Line::styled(
        app_mode,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .centered();
    // Second line contains user input, or messages to user
    let ui_text: Line;
    // Highlight error messages in red
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
