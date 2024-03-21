use crossterm::{execute, terminal::*};
use ratatui::prelude::*;
use std::io::{self, stdout, Stdout};

pub struct Tui(pub Terminal<CrosstermBackend<Stdout>>);

impl Tui {
    /// Initialize the terminal
    pub fn new() -> io::Result<Self> {
        debug!("initializing tui");
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(Self(Terminal::new(CrosstermBackend::new(stdout()))?))
    }

    /// Restore the terminal to its original state
    pub fn restore() -> io::Result<()> {
        Ok(())
    }
}

pub fn restore() {
    if let Err(err) = execute!(stdout(), LeaveAlternateScreen).and_then(|_| disable_raw_mode()) {
        error!("failed to restore tui: {err}");
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        debug!("restoring tui");
        restore();
    }
}

impl std::ops::Deref for Tui {
    type Target = Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
