use eyre::Result;

use crossterm::event;

use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph, Wrap,
    },
};

use crate::tui;

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    metadata: Option<String>,
    scroll: u16,
    exit: bool,
}

impl App {
    pub fn load_metadata(&mut self) -> Result<()> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path("./Cargo.toml")
            .exec()?;

        let content = serde_json::to_string_pretty(&metadata)?;
        std::fs::write("metadata.json", content)?;
        let string = format!("{metadata:#?}");
        self.metadata = Some(string);

        Ok(())
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            event::Event::Key(key_event) if key_event.kind == event::KeyEventKind::Press => {
                self.handle_key_event(key_event)?;
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) -> Result<()> {
        match key_event.code {
            event::KeyCode::Char('q') => self.exit(),
            event::KeyCode::Left => self.decrement_counter(),
            event::KeyCode::Right => self.increment_counter(),

            event::KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            event::KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
            event::KeyCode::PageUp => {
                let (_, page_height) = crossterm::terminal::size()?;
                self.scroll = self.scroll.saturating_sub(page_height);
            }
            event::KeyCode::PageDown => {
                let (_, page_height) = crossterm::terminal::size()?;
                self.scroll = self.scroll.saturating_add(page_height);
            }

            event::KeyCode::Char('x') => self.load_metadata()?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
            .render(area, buf);

        if let Some(metadata) = &self.metadata {
            Paragraph::new(Text::from(metadata.as_str()))
                .wrap(Wrap { trim: false })
                .scroll((self.scroll, 0))
                .block(
                    Block::default()
                        .title("Cargo Metadata")
                        .borders(Borders::ALL)
                        .border_set(border::THICK),
                )
                .render(area, buf);
        }
    }
}
