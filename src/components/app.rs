use eyre::Result;

use crossterm::event;

use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders,
    },
};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{action::Action, metadata::Features, Args};
use crate::{component::Component, metadata::PackageResolver};

#[derive(Debug)]
pub struct Icons {
    pub enabled: String,
    pub indirectly_enabled: String,
    pub disabled: String,
    pub unknown: String,
}

lazy_static::lazy_static! {
    pub static ref ICONS: Icons =
        Icons {
            // enabled: "󰄴".to_string(),
            enabled: "✓".to_string(),
            indirectly_enabled: "—".to_string(),
            // disabled: "󰝦".to_string(),
            disabled: " ".to_string(),
            unknown: "?".to_string(),
        };
}

#[derive(Debug)]
pub struct App {
    metadata: cargo_metadata::Metadata,
    tree_state: TreeState<String>,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let config_toml = args.config_toml.unwrap_or_else(|| "Cargo.toml".into());
        if !config_toml.exists() {
            eyre::bail!("{config_toml:?} not found");
        }

        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&config_toml)
            .exec()?;

        Ok(Self {
            metadata,
            tree_state: TreeState::default(),
        })
    }

    fn tree_items(&self) -> Vec<TreeItem<'static, String>> {
        let metadata = &self.metadata;
        let resolver = PackageResolver::new(metadata);
        let mut package_items = Vec::new();

        for p in &metadata.workspace_packages() {
            let mut dep_items = Vec::new();

            for (i, dep) in p.dependencies.iter().enumerate() {
                let tree_key = format!("{i}:{}:{}", p.id, dep.name);

                let Some(dep_package) = resolver.resolve_package(&p.id, &dep.name) else {
                    warn!("Could not resolve package {}", dep.name);
                    dep_items.push(TreeItem::new_leaf(
                        tree_key,
                        format!("{} {}", ICONS.unknown, dep.name),
                    ));
                    continue;
                };

                let features = Features::new(dep, dep_package);
                let active_features = features.active_features();
                let indirectly_active_features = features.indirectly_active_features();

                let mut feature_items = Vec::new();
                for (feature, feature_deps) in dep_package.features.iter() {
                    let mut spans = Vec::new();
                    if active_features.contains(feature) {
                        spans.push(Span::styled(
                            format!("{} {feature}", ICONS.enabled),
                            Style::default().green().bold(),
                        ));
                    } else if indirectly_active_features.contains(feature) {
                        spans.push(Span::styled(
                            format!("{} {feature}", ICONS.indirectly_enabled),
                            Style::default().green(),
                        ));
                    } else {
                        spans.push(Span::styled(
                            format!("{} {feature}", ICONS.disabled),
                            Style::default(),
                        ));
                    };

                    // let mut item = Vec::new();

                    if !feature_deps.is_empty() {
                        // write!(item, " ({})", feature_deps.join(", ")).expect("write failed");
                        spans.push(Span::raw(format!(" ({})", feature_deps.join(", "))));
                    }

                    let text = Text::from(Line::from(spans));

                    feature_items.push(TreeItem::new_leaf(
                        format!("{i}:{}:{}", tree_key, feature),
                        text,
                    ));
                }

                dep_items.push(
                    TreeItem::new(tree_key, dep.name.clone(), feature_items).expect("tree failed"),
                );

                // f.render_widget(Paragraph::new(p.name.as_str()), rect);
            }

            package_items.push(
                TreeItem::new(format!("{}", p.id), p.name.clone(), dep_items).expect("tree failed"),
            );
        }

        package_items
    }
}

impl Component for App {
    fn handle_key_events(&mut self, key_event: event::KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
            event::KeyCode::Up => {
                self.tree_state.key_up(&self.tree_items());
                Ok(Some(Action::Render))
            }
            event::KeyCode::Down => {
                self.tree_state.key_down(&self.tree_items());
                Ok(Some(Action::Render))
            }
            event::KeyCode::Right => {
                self.tree_state.key_right();
                // self.tree_state.selected()
                Ok(Some(Action::Render))
            }
            event::KeyCode::Left => {
                self.tree_state.key_left();
                Ok(Some(Action::Render))
            }

            // event::KeyCode::Left => self.decrement_counter(),
            // event::KeyCode::Right => self.increment_counter(),
            _ => Ok(None),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .title(Title::from(" Tree ").position(Position::Top))
            .borders(Borders::all())
            .border_style(Style::default())
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .style(Style::default());
        let items = self.tree_items();
        let tree = Tree::new(items)
            .expect("tree failed")
            .highlight_style(Style::default().on_dark_gray())
            .block(block);
        f.render_stateful_widget(tree, rect, &mut self.tree_state);
    }
}
