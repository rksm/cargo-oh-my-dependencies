use ansi_to_tui::IntoText;
use crossterm::event::{self, Event};
use eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, BorderType, Borders, Paragraph},
};

use crate::{action::Action, Args};
use crate::{args::Opt, component::Component};

use super::dependency_tab::DependencyTab;

#[derive(Debug)]
pub struct App {
    tab: DependencyTab,
    error: Option<eyre::Report>,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let Args::Omd(Opt { manifest }) = args;
        Ok(Self {
            tab: DependencyTab::new(manifest)?,
            error: None,
        })
    }

    fn render_error(&self, f: &mut Frame, rect: Rect) {
        let Some(err) = &self.error else {
            return;
        };

        let [rect] = Layout::default()
            .direction(Direction::Vertical)
            .flex(layout::Flex::Center)
            .constraints([Constraint::Max(25)])
            .areas(rect);
        let [rect] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(layout::Flex::Center)
            .constraints([Constraint::Max(100)])
            .areas(rect);
        let block = Block::default()
            .title(Title::from("Error"))
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Red))
            .title_alignment(Alignment::Center);
        let text = format!("{:?}", err)
            .into_text()
            .unwrap_or_else(|_| Text::raw(err.to_string()));
        let msg_box = Paragraph::new(text).block(block);
        f.render_widget(msg_box, rect);
    }
}

impl Component for App {
    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        self.tab.handle_events(event)
    }

    fn handle_key_events(&mut self, key_event: event::KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            event::KeyCode::Char('q') => return Action::quit(),
            event::KeyCode::Esc if self.error.is_some() => {
                self.error = None;
                return Action::render();
            }
            _ if self.error.is_some() => return Action::none(),
            _ => {}
        };

        match self.tab.handle_key_events(key_event) {
            Err(err) => {
                self.error = Some(err);
                Action::render()
            }
            action => action,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        self.tab.render(f, rect);
        self.render_error(f, rect);
    }
}
