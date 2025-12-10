use crate::App;
use count_digits::{self, CountDigits};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::cmp::max;
use unicode_display_width::width;

fn string_to_lines(
    string: &str,
    max_line_len: u16,
    first_line_num: usize,
) -> (Vec<String>, Vec<usize>) {
    let chars = string.chars();
    let mut line_numbers: Vec<usize> = Vec::new();
    let mut line_num = first_line_num;
    let mut lines: Vec<String> = Vec::new();
    let mut line: Vec<char> = Vec::new();
    let mut curr_len = 0;
    for character in chars {
        // if character is a newline, stop building line and append it and its line number to vectors
        // Increment line number and start building a new line
        if character == '\n' {
            lines.push(line.iter().collect());
            line_numbers.push(line_num);
            line_num = line_num + 1;
            line = Vec::new();
            curr_len = 0;
            continue;
        }

        // Handle tab spaces seperately due to dynamic sizing
        if character == '\t' {
            // Assume 4-space tabs
            let tab_len = 4 - (curr_len % 4);

            // Render tab spaces as tab_len number of spaces
            if curr_len + tab_len <= max_line_len as u64 {
                for _ in 0..tab_len {
                    line.push(' ');
                }
                curr_len = curr_len + tab_len;
            } else {
                lines.push(line.iter().collect());
                line_numbers.push(line_num);
                line = vec![' ', ' ', ' ', ' '];
                curr_len = 4;
            }
            continue;
        }

        // Check curr_len of line + unicode length of character vs length
        let char_width = width(&character.to_string());
        if curr_len + char_width <= max_line_len as u64 {
            // if it fits, append character to line and add its unicode length to curr len
            line.push(character);
            curr_len = curr_len + char_width;
        } else {
            // else, stop building line and append it and its line number to vectors.
            lines.push(line.iter().collect());
            line_numbers.push(line_num);

            // Start building a new line with this character
            line = vec![character];
            curr_len = char_width;
        }
    }

    // Push the last line to lines
    if curr_len > 0 {
        lines.push(line.iter().collect());
        line_numbers.push(line_num);
    }
    return (lines, line_numbers);
}

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let file_line = app.get_display_fileline();
    let scroll_amount = app.get_scroll_amount();
    let cursor_pos = app.get_cursor_pos();

    let term_size = app.get_term_size();

    let file_name = app.get_filename();
    let display_content = app.get_content();
    // Split display contents string into lines based on unicode char width, and newline characters
    let (display_lines, line_nums) = string_to_lines(display_content, term_size.1 - 2, file_line);
    let app_mode = app.get_mode();
    let ui_message = app.get_ui_display();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(2)])
        .split(frame.area());

    // Used for cursor coordinates, command line, and error messages
    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            // usize to u16 conversion and vice versa should be safe, since the number of digits in the cursor should be small
            Constraint::Length(
                5 + max(
                    (line_nums[scroll_amount + cursor_pos.0 as usize]).count_digits(),
                    cursor_pos.1.count_digits(),
                ) as u16,
            ),
            Constraint::Min(12),
        ])
        .split(layout[1]);

    // File Content
    let title = Line::from(file_name.bold());
    let content_block = Block::bordered().title(title).border_set(border::THICK);
    let mut display_content: Vec<Line> = Vec::new();
    for line in display_lines.iter() {
        let display_line = Line::styled(line, Style::default());
        display_content.push(display_line);
    }
    let display_content: Text = display_content.into();

    let content = Paragraph::new(display_content).block(content_block);
    frame.render_widget(content, layout[0]);

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
    let ui_content: Text = vec![mode_text, ui_text].into();
    let ui_block = Block::new().borders(Borders::LEFT);
    frame.render_widget(Paragraph::new(ui_content).block(ui_block), bottom_layout[1]);

    // Render cursor
    frame.set_cursor_position(Position::new(cursor_pos.1, cursor_pos.0));
}
