use std::path::PathBuf;

use crossterm::event;
use eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders,
    },
};

use crate::action::Action;
use crate::component::Component;

use super::dependency_tree::DependencyTree;
use crate::cargo;

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    DependencyTree,
}

#[derive(Debug)]
pub struct DependencyTab {
    view: View,
    config_toml: PathBuf,
    metadata: cargo_metadata::Metadata,
    dependency_tree: DependencyTree,
}

impl DependencyTab {
    pub fn new(config_toml: impl Into<PathBuf>) -> Result<Self> {
        let config_toml = config_toml.into();
        let metadata = Self::read_metadata(&config_toml)?;
        let dependency_tree = DependencyTree::new(&metadata)?;

        Ok(Self {
            config_toml,
            metadata,
            dependency_tree,
            view: Default::default(),
            // view: View::FeatureGraph {
            //     parent_package: PackageId{repr:"doppelgaenger-server 0.1.0 (path+file:///Users/robert/projects/biz/podwriter/backend/doppelgaenger-server)".to_string()},
            //     dep_name:"async-openai".to_string(),
            // },
        })
    }

    fn update(&mut self) -> Result<()> {
        self.metadata = Self::read_metadata(&self.config_toml)?;
        self.dependency_tree.update(&self.metadata);
        Ok(())
    }

    fn read_metadata(config_toml: &std::path::Path) -> Result<cargo_metadata::Metadata> {
        Ok(cargo_metadata::MetadataCommand::new()
            .manifest_path(config_toml)
            .exec()?)
    }

    fn refresh(&mut self) -> Result<Option<Action>> {
        self.update()?;
        Action::render()
    }

    fn apply_action(&mut self, action: Result<Option<Action>>) -> Result<Option<Action>> {
        match action {
            Ok(Some(Action::ShowFeatureTree {
                parent_package,
                dep_name,
            })) => {
                crate::mermaid::FeatureGraph::new(&self.metadata, &parent_package, &dep_name)
                    .build()
                    .render_and_open()?;
                Action::none()
            }

            Ok(Some(Action::ToggleFeature {
                parent_package,
                dep_name,
                feature_name,
            })) => {
                let Some(package) = self
                    .metadata
                    .packages
                    .iter()
                    .find(|p| p.id == parent_package)
                else {
                    eyre::bail!("Package not found");
                };

                cargo::EditDependency::new(package, &dep_name)
                    .toggle_feature(&feature_name)
                    .apply()?;

                self.refresh()
            }
            action => action,
        }
    }
}

impl Component for DependencyTab {
    fn handle_key_events(&mut self, key_event: event::KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            event::KeyCode::Char('q') => return Action::quit(),
            event::KeyCode::Char('g') => return self.refresh(),
            event::KeyCode::Esc => {
                self.view = View::DependencyTree;
                return Action::render();
            }
            _ => {}
        };

        self.dependency_tree
            .handle_key_events(key_event)
            .and_then(|action| self.apply_action(Ok(action)))
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
