mod controller;
mod view;
use controller::App;
use ratatui::{Terminal, prelude::Backend};
use std::fs;
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;
use view::draw_ui;

#[derive(Debug, StructOpt)]
#[structopt(name = "rust-vim")]
struct Opt {
    #[structopt(parse(from_os_str))]
    file_name: PathBuf,
}

/*
 * "main loop" which renders UI and listens for user input
 */
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    while app.running() {
        //app.update_term_size(term_size.height, term_size.width);
        terminal.draw(|frame| draw_ui(frame, app))?; // draw_ui will be a pub func from view to draw the ui
        app.handle_events()?; // controller will process inputs
    }
    return Ok(());
}

fn main() -> io::Result<()> {
    let opts = Opt::from_args();

    let mut naive_buffer = match fs::read_to_string(&opts.file_name) {
        Ok(text) => text,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => String::from(""), // File name not found, so treat it as a new file
        Err(error) => panic!("{error}"),
    };

    let mut file_name = String::from(
        opts.file_name
            .file_name()
            .expect("Error: Invalid file path provided.")
            .to_str()
            .expect("Error: File name not UTF-8 valid"),
    );

    let mut terminal = ratatui::init();
    let term_height = terminal.size()?.height;
    let term_width = terminal.size()?.width;

    let mut app = App::new(&mut file_name, &mut naive_buffer, term_height, term_width);

    let app_result = run_app(&mut terminal, &mut app);

    ratatui::restore();
    app_result
}
