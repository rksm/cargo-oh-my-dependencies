use cargo_oh_my_dependencies::{initialize_logging, run_loop, tui, Args};

fn main() {
    color_eyre::install().expect("color_eyre");
    initialize_logging().expect("initialize_logging");

    if let Err(err) = run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}

fn run() -> eyre::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        tui::restore();
        eprintln!("{info}");
    }));

    let args = <Args as clap::Parser>::parse();
    let mut terminal = tui::Tui::new()?;
    run_loop(args, &mut terminal)?;
    Ok(())
}
