mod controller;
mod model;
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
    Ok(())
}

fn main() -> io::Result<()> {
    // Read arguments and open file
    let opts = Opt::from_args();
    let file_path = opts.file_name;
    if !file_path.exists() {
        if let Some(parent) = file_path.parent()
            && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        fs::File::create(&file_path)?; // create an empty file
    }

    // build model buffer
    let mut model = model::EditorModel::new(file_path.to_str().unwrap());

    // Initialize terminal and build App structure containing app state
    let mut terminal = ratatui::init();
    let term_height = terminal.size()?.height;
    let term_width = terminal.size()?.width;
    let display_string = model.rope.to_string();
    let mut app = App::new(&mut model, display_string.as_str(), term_height, term_width);

    let app_result = run_app(&mut terminal, &mut app);

    // Restore the terminal on closure
    ratatui::restore();
    app_result
}
