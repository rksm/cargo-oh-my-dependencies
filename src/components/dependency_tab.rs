use std::path::PathBuf;

use crossterm::event::{self, Event};
use eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders,
    },
};

use crate::component::Component;
use crate::{action::Action, metadata::workspace_info::WorkspaceInfo};

use super::dependency_tree::DependencyTree;

#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    DependencyTree,
}

#[derive(Debug)]
pub struct DependencyTab {
    view: View,
    workspace_info: WorkspaceInfo,
    dependency_tree: DependencyTree,
}

impl DependencyTab {
    pub fn new(config_toml: impl Into<PathBuf>) -> Result<Self> {
        let workspace_info = WorkspaceInfo::load(config_toml)?;
        let dependency_tree = DependencyTree::new(&workspace_info)?;

        Ok(Self {
            workspace_info,
            dependency_tree,
            view: Default::default(),
            // view: View::FeatureGraph {
            //     parent_package: PackageId{repr:"doppelgaenger-server 0.1.0 (path+file:///Users/robert/projects/biz/podwriter/backend/doppelgaenger-server)".to_string()},
            //     dep_name:"async-openai".to_string(),
            // },
        })
    }

    fn update(&mut self) -> Result<()> {
        self.workspace_info.update()?;
        self.dependency_tree.update(&self.workspace_info);
        Ok(())
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
                crate::mermaid::FeatureGraph::new(&self.workspace_info, &parent_package, &dep_name)
                    .build()
                    .render_and_open()?;
                Action::none()
            }

            Ok(Some(Action::ToggleFeature {
                parent_package,
                dep_name,
                feature_name,
            })) => {
                self.workspace_info
                    .toggle_feature(&parent_package, &dep_name, &feature_name)?;
                self.refresh()
            }
            action => action,
        }
    }
}

impl Component for DependencyTab {
    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        match event {
            Event::Key(key_event) => self.handle_key_events(key_event),
            Event::Mouse(mouse_event) => self.handle_mouse_events(mouse_event),
            Event::Resize(..) => Action::render(),
            _ => Ok(None),
        }
    }

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
        let location = self.dependency_tree.location();

        let breadcrumbs = location
            .as_ref()
            .map(|l| l.breadcrumbs())
            .unwrap_or_default();
        let help = location.as_ref().map(|l| l.help()).unwrap_or_default();

        let block = Block::default()
            .title(Title::from(breadcrumbs).position(Position::Top))
            .title_bottom(help)
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
