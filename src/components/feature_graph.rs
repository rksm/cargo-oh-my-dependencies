use std::collections::HashMap;

use cargo_metadata::PackageId;
use eyre::Result;
use ratatui::widgets::{BorderType, Paragraph, Wrap};

use crossterm::event;
use ratatui::prelude::*;

use tui_nodes::*;

use crate::action::Action;
use crate::component::Component;
use crate::metadata::PackageResolver;

#[derive(Debug)]
pub struct FeatureGraph {
    metadata: cargo_metadata::Metadata,
    package_id: PackageId,
    dep_name: String,
}

impl FeatureGraph {
    pub fn new(d: &cargo_metadata::Metadata, package_id: PackageId, dep_name: String) -> Self {
        Self {
            metadata: d.clone(),
            package_id,
            dep_name,
        }
    }
}

impl Component for FeatureGraph {
    fn handle_key_events(&mut self, _key_event: event::KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    fn render(&mut self, f: &mut Frame, space: Rect) {
        // let package = self
        //     .metadata
        //     .workspace_packages()
        //     .iter()
        //     .find(|p| p.id == self.package_id)
        //     .unwrap();
        let resolver = PackageResolver::new(&self.metadata);
        let Some(dep_package) = resolver.resolve_package(&self.package_id, &self.dep_name) else {
            unimplemented!("Could not resolve package: {:?}", self.dep_name);
        };

        let mut node_indexes = HashMap::new();
        let mut nodes_a = Vec::new();
        let mut nodes_b = Vec::new();
        let mut connections = Vec::new();
        let mut connections_b = Vec::new();
        let max_width = 50.min(space.width) as usize;

        for (feature, dep_features) in &dep_package.features {
            let from_index = *node_indexes.entry(feature).or_insert_with(|| {
                let index = nodes_a.len();
                let width = (feature.len() + 2).min(max_width) as u16;
                nodes_a.push(NodeLayout::new((width, 5)));
                nodes_b.push(NodeLayout::new((width, 5)));
                index
            });

            for dep_feature in dep_features {
                let to_node_index = *node_indexes.entry(dep_feature).or_insert_with(|| {
                    let index = nodes_a.len();
                    let width = (dep_feature.len() + 2).min(max_width) as u16;
                    nodes_a.push(NodeLayout::new((width, 5)));
                    nodes_b.push(NodeLayout::new((width, 5)));
                    index
                });

                debug!(
                    "{} ({}) -> {} ({})",
                    feature, from_index, dep_feature, to_node_index
                );

                connections.push(Connection::new(from_index, 0, to_node_index, 0));
                connections_b.push(Connection::new(from_index, 0, to_node_index, 0));
            }
        }

        // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

        let mut graph = NodeGraph::new(
            nodes_a,
            connections,
            space.width as usize,
            space.height as usize,
        );
        graph.calculate();

        let index_to_node = node_indexes
            .into_iter()
            .map(|(k, v)| (v, k))
            .collect::<HashMap<_, _>>();
        let zones = graph.split(space);
        for (idx, ea_zone) in zones.into_iter().enumerate() {
            let node_name = index_to_node.get(&idx).unwrap();
            let t = Paragraph::new(node_name.as_str()).wrap(Wrap { trim: false });
            f.render_widget(t, ea_zone);
        }

        f.render_stateful_widget(graph, space, &mut ());

        // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

        // let mut graph = NodeGraph::new(
        //     nodes_a,
        //     connections,
        //     space.width as usize,
        //     space.height as usize,
        // );
        // graph.calculate();

        // let zones = graph.split(space);
        // let mut min_x = space.width;
        // let mut max_x = 0;
        // let mut min_y = space.height;
        // let mut max_y = 0;

        // for (idx, ea_zone) in zones.into_iter().enumerate() {
        //     min_x = min_x.min(ea_zone.x);
        //     max_x = max_x.max(ea_zone.x + ea_zone.width);
        //     min_y = min_y.min(ea_zone.y);
        //     max_y = max_y.max(ea_zone.y + ea_zone.height);
        // }

        // let actual_width = (max_x - min_x) + 6;
        // let actual_height = (max_y - min_y) + 6;

        // let [inner] = Layout::default()
        //     .flex(layout::Flex::Center)
        //     .direction(Direction::Vertical)
        //     .constraints([Constraint::Length(actual_height)].as_ref())
        //     .margin(1)
        //     .areas(space);
        // let [inner] = Layout::default()
        //     .flex(layout::Flex::Center)
        //     .direction(Direction::Horizontal)
        //     .constraints([Constraint::Length(actual_width)].as_ref())
        //     .margin(1)
        //     .areas(inner);

        // let mut graph = NodeGraph::new(
        //     nodes_b,
        //     connections_b,
        //     actual_width as usize,
        //     actual_height as usize,
        // );
        // graph.calculate();

        // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

        // let index_to_node = node_indexes
        //     .into_iter()
        //     .map(|(k, v)| (v, k))
        //     .collect::<HashMap<_, _>>();
        // let zones = graph.split(inner);
        // for (idx, ea_zone) in zones.into_iter().enumerate() {
        //     let node_name = index_to_node.get(&idx).unwrap();
        //     let t = Paragraph::new(node_name.as_str()).wrap(Wrap { trim: false });
        //     f.render_widget(t, ea_zone);
        // }

        // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

        // f.render_stateful_widget(graph, inner, &mut ());
    }
}
