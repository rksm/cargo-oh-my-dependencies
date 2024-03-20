#[macro_use]
extern crate tracing;

mod app;
mod logging;
mod tui;

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

fn main() {
    color_eyre::install().expect("color_eyre");
    logging::initialize_logging().expect("initialize_logging");

    if let Err(err) = run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

fn run() -> eyre::Result<()> {
    info!("Starting up...");

    // Need to use catch_unwind here to always restore the terminal
    let result = std::panic::catch_unwind(|| {
        let mut terminal = tui::init()?;
        let mut app = app::App::default();
        app.run(&mut terminal)
    });

    tui::restore()?;

    result.map_err(|_| eyre::eyre!("Panic in the main thread"))?
}
