use crossterm::event::{Event, KeyEvent, MouseEvent};
use eyre::Result;
use ratatui::{layout::Rect, Frame};

use crate::action::Action;

pub trait Component {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        match event {
            Event::Key(key_event) => self.handle_key_events(key_event),
            Event::Mouse(mouse_event) => self.handle_mouse_events(mouse_event),
            Event::Resize(x, y) => Ok(Some(Action::Resize(x, y))),
            _ => Ok(None),
        }
    }

    fn handle_key_events(&mut self, _key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    fn handle_mouse_events(&mut self, _mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    fn update(&mut self, _action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame, rect: Rect);
}
