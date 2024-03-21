use crossterm::event;

use crate::{action::Action, app, component::Component, tui};

/// runs the application's main loop until the user quits
pub fn run_loop(terminal: &mut tui::Tui) -> eyre::Result<()> {
    let mut app = app::App::default();

    loop {
        let bounds = terminal.size()?;
        terminal.draw(|frame| {
            app.render(frame, bounds);
        })?;

        loop {
            let event = event::read()?;
            info!("event: {:?}", event);
            let mut action = Component::handle_events(&mut app, event)?;
            let mut render = false;
            while let Some(this_action) = action.take() {
                let quit = matches!(this_action, Action::Quit);
                render = matches!(this_action, Action::Render);
                action = app.update(this_action)?;
                if quit {
                    return Ok(());
                }
            }
            if render {
                break;
            }
        }
    }
}
