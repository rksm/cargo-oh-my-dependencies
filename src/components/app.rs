use crossterm::event;
use eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders,
    },
};

use crate::component::Component;
use crate::{action::Action, Args};

use super::dependency_tree::DependencyTree;

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    DependencyTree,
}

#[derive(Debug)]
pub struct App {
    view: View,
    metadata: cargo_metadata::Metadata,
    dependency_tree: DependencyTree,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let mut config_toml = args.config_toml.unwrap_or_else(|| "Cargo.toml".into());
        if config_toml.is_dir() {
            config_toml.push("Cargo.toml");
        }
        if !config_toml.exists() {
            eyre::bail!("{config_toml:?} not found");
        }

        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&config_toml)
            .exec()?;

        let dependency_tree = DependencyTree::new(&metadata)?;

        Ok(Self {
            view: Default::default(),
            // view: View::FeatureGraph {
            //     parent_package: PackageId{repr:"doppelgaenger-server 0.1.0 (path+file:///Users/robert/projects/biz/podwriter/backend/doppelgaenger-server)".to_string()},
            //     dep_name:"async-openai".to_string(),
            // },
            metadata,
            dependency_tree,
        })
    }
}

impl Component for App {
    fn handle_key_events(&mut self, key_event: event::KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            event::KeyCode::Char('q') => return Ok(Some(Action::Quit)),
            event::KeyCode::Esc => {
                self.view = View::DependencyTree;
                return Ok(Some(Action::Render));
            }
            _ => {}
        };

        let action = self.dependency_tree.handle_key_events(key_event);

        if let Ok(Some(Action::ShowFeatureTree {
            parent_package,
            dep_name,
        })) = action
        {
            crate::mermaid::FeatureGraph::new(&self.metadata, &parent_package, &dep_name)
                .build()
                .render_and_open()?;
            return Ok(None);
        }

        action
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .title(Title::from(" Tree ").position(Position::Top))
            .borders(Borders::all())
            .border_style(Style::default())
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default());
        block.render(rect, f.buffer_mut());

        let [inner] = Layout::default()
            .flex(layout::Flex::Center)
            // .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(1)
            .areas(rect);

        self.dependency_tree.render(f, inner);
    }
}
