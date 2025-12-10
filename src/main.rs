mod controller;
mod view;
use controller::App;
use ratatui::{Terminal, prelude::Backend};
use std::io;
use view::draw_ui;

/*
 * "main loop" which renders UI and listens for user input
 */
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    while app.running() {
        let term_size = terminal.size()?;
        app.update_term_size(term_size.height, term_size.width);
        terminal.draw(|frame| draw_ui(frame, app))?; // draw_ui will be a pub func from view to draw the ui
        app.handle_events()?; // controller will process inputs
    }
    return Ok(());
}

fn main() -> io::Result<()> {
    let mut dummy_file_name = String::from("Bee_Movie.txt");
    let mut dummy_string = String::from(
        "According to all known laws of aviation, there is no way a bee should be able to fly.
        Its wings are too small to get its fat little body off the ground.
        The bee, of course, flies anyway because bees don't care what humans think is impossible.
        Yellow, black. Yellow, black. Yellow, black. Yellow, black.
        Ooh, black and yellow!
        Let's shake it up a little.
        Barry! Breakfast is ready!
        Coming!
        Hang on a second.
        Hello?
        Barry?
        Adam?
        Can you believe this is happening?
        I can't.
        I'll pick you up.
        Looking sharp.
        Use the stairs, Your father paid good money for those.
        Sorry. I'm excited.
        Here's the graduate.
        We're very proud of you, son.",
    );

    let mut terminal = ratatui::init();
    let term_height = terminal.size()?.height;
    let term_width = terminal.size()?.width;

    let mut app = App::new(
        &mut dummy_file_name,
        &mut dummy_string,
        term_height,
        term_width,
    );

    let app_result = run_app(&mut terminal, &mut app);

    ratatui::restore();
    app_result
}
