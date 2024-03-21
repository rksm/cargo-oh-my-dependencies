use eyre::Result;

use crossterm::event;

use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph,
    },
};

use crate::action::Action;
use crate::component::Component;

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
}

impl App {
    fn increment_counter(&mut self) -> Result<Option<Action>> {
        self.counter += 1;
        Ok(Some(Action::Render))
    }

    fn decrement_counter(&mut self) -> Result<Option<Action>> {
        self.counter -= 1;
        Ok(Some(Action::Render))
    }
}

impl Component for App {
    fn handle_key_events(&mut self, key_event: event::KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
            event::KeyCode::Left => self.decrement_counter(),
            event::KeyCode::Right => self.increment_counter(),
            _ => Ok(None),
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, f.buffer_mut());
    }
}
