mod app;
mod tui;

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

fn main() {
    color_eyre::install().expect("color_eyre");
    env_logger::init();

    if let Err(err) = run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

fn run() -> eyre::Result<()> {
    // Need to use catch_unwind here to always restore the terminal
    let result = std::panic::catch_unwind(|| {
        let mut terminal = tui::init()?;
        let mut app = app::App::default();
        app.run(&mut terminal)
    });

    tui::restore()?;

    result.map_err(|_| eyre::eyre!("Panic in the main thread"))?
}
