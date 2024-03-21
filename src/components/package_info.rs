use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::component::Component;

pub struct PackageInfo<'a> {
    pub metadata: &'a cargo_metadata::Metadata,
}

impl<'a> PackageInfo<'a> {
    pub fn new(metadata: &'a cargo_metadata::Metadata) -> Self {
        Self { metadata }
    }
}

impl<'a> Component for PackageInfo<'a> {
    fn render(&mut self, f: &mut ratatui::prelude::Frame, rect: ratatui::prelude::Rect) {
        let name = &self.metadata.workspace_default_members[0].to_string();

        let paragraph = Paragraph::new(name.as_str())
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .title("Package")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .right_aligned()
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, rect)
    }
}
