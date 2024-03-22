use cargo_metadata::PackageId;
use crossterm::event;
use eyre::Result;
use ratatui::prelude::*;
use std::collections::HashMap;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::{action::Action, metadata::Features};
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

#[derive(Debug, Clone)]
enum Item {
    WorkspacePackage(PackageId),
    Dependency(String),
    Feature(String),
}

#[derive(Debug)]
pub struct DependencyTree {
    tree_state: TreeState<String>,
    items: Vec<TreeItem<'static, String>>,
    indexed_items: HashMap<String, Item>,
}

impl DependencyTree {
    pub fn new(d: &cargo_metadata::Metadata) -> Result<Self> {
        let (items, indexed_items) = Self::tree_items(d);
        Ok(Self {
            tree_state: TreeState::default(),
            items,
            indexed_items,
        })
    }

    fn tree_items(
        metadata: &cargo_metadata::Metadata,
    ) -> (Vec<TreeItem<'static, String>>, HashMap<String, Item>) {
        let resolver = PackageResolver::new(metadata);
        let mut package_items = Vec::new();
        let mut indexed_items = HashMap::new();

        for p in &metadata.workspace_packages() {
            let mut dep_items = Vec::new();

            for (i, dep) in p.dependencies.iter().enumerate() {
                let tree_key = format!("{i}:{}:{}", p.id, dep.name);

                let Some(dep_package) = resolver.resolve_package(&p.id, &dep.name) else {
                    warn!("Could not resolve package {}", dep.name);
                    indexed_items.insert(tree_key.clone(), Item::Dependency(dep.name.clone()));
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
                            Style::default().white(),
                        ));
                    };

                    // let mut item = Vec::new();

                    if !feature_deps.is_empty() {
                        // write!(item, " ({})", feature_deps.join(", ")).expect("write failed");
                        spans.push(Span::raw(format!(" ({})", feature_deps.join(", "))));
                    }

                    let text = Text::from(Line::from(spans));
                    let key = format!("{tree_key}:{feature}");
                    indexed_items.insert(key.clone(), Item::Feature(feature.clone()));
                    feature_items.push(TreeItem::new_leaf(key, text));
                }

                indexed_items.insert(tree_key.clone(), Item::Dependency(dep.name.clone()));
                let span = Span::styled(dep.name.clone(), Style::default().white());
                dep_items.push(TreeItem::new(tree_key, span, feature_items).expect("tree failed"));

                // f.render_widget(Paragraph::new(p.name.as_str()), rect);
            }

            indexed_items.insert(p.id.to_string(), Item::WorkspacePackage(p.id.clone()));
            let span = Span::styled(p.name.clone(), Style::default().white().bold());
            package_items
                .push(TreeItem::new(p.id.to_string(), span, dep_items).expect("tree failed"));
        }

        (package_items, indexed_items)
    }
}

impl Component for DependencyTree {
    fn handle_key_events(&mut self, key_event: event::KeyEvent) -> Result<Option<Action>> {
        match key_event.code {
            event::KeyCode::Char('q') => Ok(Some(Action::Quit)),
            event::KeyCode::Up => {
                self.tree_state.key_up(&self.items);
                Ok(Some(Action::Render))
            }
            event::KeyCode::Down => {
                self.tree_state.key_down(&self.items);
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

            event::KeyCode::Enter => {
                let selected = self
                    .tree_state
                    .selected()
                    .into_iter()
                    .filter_map(|s| self.indexed_items.get(&s))
                    .collect::<Vec<_>>();

                match &selected[..] {
                    [Item::WorkspacePackage(id), Item::Dependency(name)] => {
                        Ok(Some(Action::ShowFeatureTree {
                            parent_package: id.clone(),
                            dep_name: name.clone(),
                        }))
                    }

                    _ => todo!(),
                }
            }
            _ => Ok(None),
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let tree = Tree::new(self.items.clone())
            .expect("tree failed")
            .highlight_style(Style::default().on_dark_gray());

        f.render_stateful_widget(tree, rect, &mut self.tree_state);
    }
}
