mod view;
use ratatui;
use std::io;
pub use view::View;

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

    let app_result = View::new(&mut dummy_file_name, &mut dummy_string).run(&mut terminal);

    ratatui::restore();
    app_result
}
