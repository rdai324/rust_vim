use crate::{App, controller::Mode};
use count_digits::{self, CountDigits};
use crossterm::{cursor::SetCursorStyle, execute};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Position},
    prelude::Rect,
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::cmp::max;
use std::io::stdout;

const LEFT_HELP_TEXT: &str = "Normal Mode:
Move the cursor with arrow keys
[i] to start editing text in Insertion Mode
[:] to start typing in Command Mode
[/] to start a query in Search Input Mode

Command Mode:
[Esc] to cancel and return to Normal Mode
[Enter] to submit the command
Commands:
:q => Quit editing
:w => Write to file
:wq => Write to file, then quit
:num => Toggle line numbers";

const RIGHT_HELP_TEXT: &str = "Insertion Mode:
Move the cursor with arrow keys
Type to insert characters at the cursor location
[Backspace] to delete characters at the cursor location
[Esc] to return to Normal Mode

Search Input Mode:
[Esc] to cancel and return to Normal Mode
[Enter] to submit the search query and enter Search Mode

Search Mode:
[Esc] to cancel and return to Normal Mode
[n] to jump to the next match
[p] to jump to the previous match";

pub const MAX_HELP_SCROLL: u16 = 14;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let file_name = app.get_filename();
    let display_lines = app.get_content();
    let show_line_num = app.get_show_line_num();
    let search_term = app.get_search_term();

    let mode_text = app.get_mode_text();
    let ui_message = app.get_msg_display();

    let app_mode = app.get_app_mode();
    let scroll_amount = app.get_scroll_amount();
    let help_scroll = app.get_scroll_help_amount();
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
        mode_text,
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
            Line::styled(ui_message, Style::default().fg(Color::Black).bg(Color::Red)).centered();
    } else {
        ui_text = Line::styled(ui_message, Style::default().fg(Color::Black)).centered();
    }
    let ui_content: Text = vec![mode_text, ui_text].into();
    let ui_block = Block::new().borders(Borders::LEFT);
    frame.render_widget(Paragraph::new(ui_content).block(ui_block), bottom_layout[1]);

    // Render Help pop-up if in Help mode
    if let Mode::Help = app_mode {
        let help_popup_block = Block::bordered()
            .title("Help Menu")
            .border_set(border::THICK)
            .style(Style::default().bg(Color::Blue));
        let area = frame.area();
        let area = popup_area(area, 90, 75);
        frame.render_widget(Clear, area);
        frame.render_widget(help_popup_block, area);

        let help_popup_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let left_help_text = Paragraph::new(LEFT_HELP_TEXT)
            .wrap(Wrap { trim: false })
            .scroll((help_scroll, 0));
        let right_help_text = Paragraph::new(RIGHT_HELP_TEXT)
            .wrap(Wrap { trim: false })
            .scroll((help_scroll, 0));
        frame.render_widget(left_help_text, help_popup_chunks[0]);
        frame.render_widget(right_help_text, help_popup_chunks[1]);
    }

    // Render cursor
    if let Mode::Insert = app_mode {
        execute!(stdout(), SetCursorStyle::BlinkingBar).unwrap();
    } else {
        execute!(stdout(), SetCursorStyle::BlinkingBlock).unwrap();
    }
    frame.set_cursor_position(Position::new(cursor_pos.1, cursor_pos.0));
}

// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
