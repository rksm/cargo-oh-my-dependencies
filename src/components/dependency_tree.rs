use cargo_metadata::PackageId;
use crossterm::event;
use eyre::Result;
use ratatui::prelude::*;
use std::collections::HashMap;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use crate::action::Action;
use crate::component::Component;
use crate::metadata::dep_tree::{self, DepTree, DepTreeNode};
use crate::metadata::workspace_info::WorkspaceInfo;

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
            enabled: "✓".to_string(),
            indirectly_enabled: "—".to_string(),
            disabled: " ".to_string(),
            unknown: "?".to_string(),
        };
}

#[derive(Debug, Clone)]
pub enum Location {
    Package(PackageId),
    Dependency((PackageId, String)),
    Feature((PackageId, String, String)),
}

impl Location {
    pub fn id(&self) -> String {
        let id = match self {
            Location::Package(id) => id,
            Location::Dependency((id, _)) => id,
            Location::Feature((id, _, _)) => id,
        };
        id.to_string().replace("path+file://", "")
    }

    pub fn breadcrumbs(&self) -> Vec<Span<'static>> {
        match self {
            Location::Package(_) => {
                vec![
                    Span::raw(" "),
                    Span::styled(self.id(), Style::default().bold()),
                    Span::raw(" "),
                ]
            }
            Location::Dependency((_, name)) => {
                vec![
                    Span::raw(" "),
                    Span::styled(self.id(), Style::default().bold()),
                    Span::raw(" > "),
                    Span::styled(name.clone(), Style::default().bold()),
                    Span::raw(" "),
                ]
            }

            Location::Feature((_, name, feature_name)) => {
                vec![
                    Span::raw(" "),
                    Span::styled(self.id(), Style::default().bold()),
                    Span::raw(" > "),
                    Span::styled(name.clone(), Style::default().bold()),
                    Span::raw(" > "),
                    Span::styled(feature_name.clone(), Style::default().bold()),
                    Span::raw(" "),
                ]
            }
        }
    }

    pub fn help(&self) -> Vec<Span<'static>> {
        let mut help = Vec::new();

        help.push("r".blue());
        help.push("efresh".dim());
        help.push(" ".dim());
        help.push("q".blue());
        help.push("uit".dim());
        help.push(" ".dim());

        match self {
            Location::Package(_) => {
                help.insert(0, "raph ".dim());
                help.insert(0, "g".blue());
                help.insert(0, " ".dim());
                help
            }
            Location::Dependency((_, _)) => {
                help.insert(0, " ".dim());
                help
            }

            Location::Feature((_, _, _)) => {
                help.insert(0, " ".dim());
                help.insert(0, "<enter>".blue());
                help.insert(0, "toggle".dim());
                help.insert(0, " ".dim());
                help.insert(0, " ".dim());
                help
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct DependencyTree {
    tree_state: TreeState<String>,
    items: Vec<TreeItem<'static, String>>,

    tree: dep_tree::DepTree,
    tree_index: HashMap<String, usize>,
}

impl DependencyTree {
    pub fn new(d: &WorkspaceInfo) -> Result<Self> {
        let mut me = Self::default();
        me.update(d);
        Ok(me)
    }

    pub fn update(&mut self, info: &WorkspaceInfo) {
        let (items, indexed_items, tree) = Self::tree_items(info);
        info!(
            "updated dependency tree with {} items ({} root nodes)",
            indexed_items.len(),
            items.len()
        );
        self.items = items;
        self.tree_index = indexed_items;
        self.tree = tree;
    }

    pub fn location(&self) -> Option<Location> {
        use DepTreeNode::*;

        let selected = self
            .tree_state
            .selected()
            .into_iter()
            .filter_map(|s| {
                self.tree_index
                    .get(&s)
                    .and_then(|i| self.tree.items.get(*i))
            })
            .collect::<Vec<_>>();

        match &selected[..] {
            [WorkspacePackage { id, .. }] => Some(Location::Package(id.clone())),
            [WorkspacePackage { id, .. }, Dependency { name, .. }] => {
                Some(Location::Dependency((id.clone(), name.clone())))
            }
            [WorkspacePackage { id, .. }, Dependency { name, .. }, Feature {
                name: feature_name, ..
            }] => Some(Location::Feature((
                id.clone(),
                name.clone(),
                feature_name.clone(),
            ))),
            _ => None,
        }
    }

    fn tree_items(
        workspace_info: &WorkspaceInfo,
    ) -> (
        Vec<TreeItem<'static, String>>,
        HashMap<String, usize>,
        DepTree,
    ) {
        let mut index = HashMap::new();
        let tree = workspace_info.tree();
        let items = tree.visit_post_order(&mut |node, i, children| {
            use DepTreeNode::*;

            match (node, children) {
                (WorkspacePackage { id, .. }, Some(children)) => {
                    let key = node.widget_id();
                    index.insert(key.clone(), i);
                    let workspace_packages = &workspace_info.workspace_packages();
                    let p = workspace_packages.iter().find(|&&p| &p.id == id).unwrap();
                    let span = Span::styled(p.name.clone(), Style::default().white().bold());
                    TreeItem::new(id.to_string(), span, children).expect("tree failed")
                }

                (UnresolvedDependency { name, kind }, None) => {
                    let key = format!("{i}:{}", node.widget_id());
                    index.insert(key.clone(), i);
                    let icon = &ICONS.unknown;
                    let label = match kind {
                        cargo_metadata::DependencyKind::Normal => format!("{icon} {name}"),
                        _ => format!("{icon} {name} ({kind})"),
                    };
                    TreeItem::new_leaf(key, label)
                }

                (Dependency { name, kind, .. }, Some(children)) => {
                    let key = format!("{i}:{}", node.widget_id());
                    index.insert(key.clone(), i);
                    let label = match kind {
                        cargo_metadata::DependencyKind::Normal => name.clone(),
                        _ => format!("{name} ({kind})"),
                    };
                    let span = Span::styled(label, Style::default().white());
                    TreeItem::new(key, span, children).expect("tree failed")
                }

                (Feature { name, status, deps }, None) => {
                    use dep_tree::FeatureStatus::*;

                    let mut spans = Vec::new();

                    match status {
                        Enabled => spans.push(Span::styled(
                            format!("{} {name}", ICONS.enabled),
                            Style::default().green().bold(),
                        )),
                        IndirectlyEnabled => spans.push(Span::styled(
                            format!("{} {name}", ICONS.indirectly_enabled),
                            Style::default().green(),
                        )),
                        Disabled => spans.push(Span::styled(
                            format!("{} {name}", ICONS.disabled),
                            Style::default().white(),
                        )),
                    }

                    if !deps.is_empty() {
                        spans.push(Span::raw(format!(" ({})", deps.join(", "))));
                    }

                    let text = Text::from(Line::from(spans));
                    let key = format!("{i}:{}", node.widget_id());
                    index.insert(key.clone(), i);
                    TreeItem::new_leaf(key, text)
                }

                _ => unreachable!(),
            }
        });

        (items, index, tree)
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
                use DepTreeNode::*;

                let selected = self
                    .tree_state
                    .selected()
                    .into_iter()
                    .filter_map(|s| {
                        self.tree_index
                            .get(&s)
                            .and_then(|i| self.tree.items.get(*i))
                    })
                    .collect::<Vec<_>>();

                match &selected[..] {
                    [WorkspacePackage { id, .. }, Dependency { name, .. }] => {
                        Ok(Some(Action::ShowFeatureTree {
                            parent_package: id.clone(),
                            dep_name: name.clone(),
                        }))
                    }

                    [WorkspacePackage { id, .. }, Dependency { name, kind, .. }, Feature {
                        name: feature_name,
                        status,
                        ..
                    }] => Ok(Some(Action::ToggleFeature {
                        parent_package: id.clone(),
                        dep_name: name.clone(),
                        dep_kind: *kind,
                        feature_name: feature_name.clone(),
                        feature_status: status.clone(),
                    })),

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
