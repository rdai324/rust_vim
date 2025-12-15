use crate::{
    App,
    controller::{Mode, QuitSelection},
};
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
Move the cursor with arrow keys or hjkl
[i] to start editing text in Insertion Mode
[:] to start typing in Command Mode
[/] to start a query in Search Input Mode
[Esc] to turn off Search Highlights

Command Mode:
[Esc] to cancel and return to Normal Mode
[Enter] to submit the command
Commands:
:q => Quit editing
:w => Write to file
:wq => Write to file, then quit
:num => Toggle line numbers
:dd => Delete current line of file";

const RIGHT_HELP_TEXT: &str = "Insertion Mode:
Move the cursor with arrow keys
Type to insert characters at the cursor location
[Enter] to insert a new line
[Backspace] to delete characters left of the cursor location
[Del] to delete characters right of the cursor location
[Esc] to return to Normal Mode

Search Input Mode:
[Esc] to cancel and return to Normal Mode
[Enter] to submit the search query and highlight matches";

pub const MAX_HELP_SCROLL: u16 = 14;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    // For the main content section of UI
    let file_name = app.get_filename();
    let display_lines = app.get_content();
    let show_line_num = app.get_show_line_num();
    let show_highlights = app.get_show_highlights();

    // For message bar of UI
    let mode_text = app.get_mode_text();
    let ui_message = app.get_msg_display();

    // For cursor location section of UI
    let scroll_amount = app.get_scroll_amount();
    let cursor_pos = app.get_cursor_pos();
    let curr_row = display_lines[(scroll_amount + cursor_pos.0) as usize - 1].line_num;
    let curr_col = app.get_cursor_inline_index();

    // Other important items used for View UI
    let app_mode = app.get_app_mode();
    let help_scroll = app.get_scroll_help_amount();
    let quit_selection = app.get_quit_selection();

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
        let mut display_line = vec![];

        // Format line numbers with yellow color
        let mut line_content_index = 0;
        if show_line_num {
            line_content_index = line.line_content.find('|').unwrap();
            display_line.push(Span::styled(
                &line.line_content[..line_content_index],
                Style::default().fg(Color::Yellow),
            ));
        }

        // Highlight search matches if present
        if show_highlights {
            let mut curr_index = line_content_index;
            // Iterate over the line's highlighted ranges
            for highlight_range in &line.highlight_ranges {
                // normal white text (not search match)
                display_line.push(Span::raw(
                    &line.line_content[curr_index..highlight_range.start],
                ));
                // highlighted range
                display_line.push(Span::styled(
                    &line.line_content[highlight_range.start..highlight_range.end],
                    Style::default().fg(Color::White).bg(Color::Cyan),
                ));
                curr_index = highlight_range.end;
            }
            // Add last substring as normal text
            display_line.push(Span::raw(&line.line_content[curr_index..]));
        } else {
            // No search matches, display as normal white text
            display_line.push(Span::styled(
                &line.line_content[line_content_index..],
                Style::default(),
            ));
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
    // Highlight error messages in red
    let ui_text: Line = if ui_message.starts_with("Error") {
        Line::styled(ui_message, Style::default().bg(Color::Red)).centered()
    } else {
        Line::styled(ui_message, Style::default()).centered()
    };
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

        // Split help popup into left half and right half
        let help_popup_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .spacing(1)
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

    // Render Quit popup if confirming user's intent to quit without saving
    if let Mode::Quit = app_mode {
        let quit_popup_block = Block::bordered()
            .border_set(border::THICK)
            .style(Style::default().bg(Color::DarkGray));
        let area = frame.area();
        let area = popup_area(area, 90, 60);
        frame.render_widget(Clear, area);
        frame.render_widget(quit_popup_block, area);

        let quit_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(15), Constraint::Min(5)])
            .split(area);

        // Header Text of quit popup
        let header_text = format!("Quit without saving to {file_name}?");
        frame.render_widget(
            Paragraph::new(header_text)
                .centered()
                .style(Style::default().add_modifier(Modifier::BOLD)),
            quit_layout[0],
        );

        // Three selection segments corresponding to the three quit options
        let selection_layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .spacing(1)
            .split(quit_layout[1]);

        // Boxes for each of the options in their unselected form
        let mut cancel_box = Paragraph::new("Cancel")
            .block(Block::bordered().border_set(border::ROUNDED))
            .centered()
            .style(Style::default().fg(Color::LightYellow).bg(Color::DarkGray));
        let mut quit_box = Paragraph::new("Quit Without Saving")
            .block(Block::bordered().border_set(border::ROUNDED))
            .centered()
            .style(Style::default().fg(Color::LightRed).bg(Color::DarkGray));
        let mut save_and_quit_box = Paragraph::new("Save & Quit")
            .block(Block::bordered().border_set(border::ROUNDED))
            .centered()
            .style(Style::default().fg(Color::LightGreen).bg(Color::DarkGray));

        // Update styling for the selected box to make it more vibrant
        match quit_selection {
            QuitSelection::Cancel => {
                cancel_box = cancel_box.block(Block::bordered().border_set(border::THICK));
                cancel_box = cancel_box.style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            }
            QuitSelection::NoSaveQuit => {
                quit_box = quit_box.block(Block::bordered().border_set(border::THICK));
                quit_box = quit_box.style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                );
            }
            QuitSelection::SaveAndQuit => {
                save_and_quit_box =
                    save_and_quit_box.block(Block::bordered().border_set(border::THICK));
                save_and_quit_box = save_and_quit_box.style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                );
            }
        }

        frame.render_widget(cancel_box, selection_layout[2]);
        frame.render_widget(quit_box, selection_layout[1]);
        frame.render_widget(save_and_quit_box, selection_layout[0]);
    }

    // Render cursor if not in pop-up modes
    match app_mode {
        Mode::Normal | Mode::Command | Mode::SearchInput => {
            execute!(stdout(), SetCursorStyle::BlinkingBlock).unwrap();
            frame.set_cursor_position(Position::new(cursor_pos.1, cursor_pos.0));
        }
        Mode::Insert => {
            execute!(stdout(), SetCursorStyle::BlinkingBar).unwrap();
            frame.set_cursor_position(Position::new(cursor_pos.1, cursor_pos.0));
        }
        _ => {}
    }
}

// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
