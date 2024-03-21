use cargo_oh_my_dependencies::{initialize_logging, run_loop, tui};

fn main() {
    color_eyre::install().expect("color_eyre");
    initialize_logging().expect("initialize_logging");

    if let Err(err) = run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

fn run() -> eyre::Result<()> {
    // Need to use catch_unwind here to always restore the terminal
    let result = std::panic::catch_unwind(|| {
        let mut terminal = tui::init()?;
        run_loop(&mut terminal)
    });

    tui::restore()?;

    result.map_err(|err| {
        let err = err.downcast_ref::<&str>().unwrap_or(&"unknown error");
        eyre::eyre!("panic: {err}")
    })?
}
