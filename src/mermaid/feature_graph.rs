use std::collections::HashMap;

use cargo_metadata::PackageId;

use crate::metadata::PackageResolver;

use super::code_gen::{Edge, Graph, Node};

#[derive(Debug)]
pub struct FeatureGraph<'a> {
    metadata: &'a cargo_metadata::Metadata,
    package_id: &'a PackageId,
    dep_name: &'a String,
}

impl<'a> FeatureGraph<'a> {
    pub fn new(
        metadata: &'a cargo_metadata::Metadata,
        package_id: &'a PackageId,
        dep_name: &'a String,
    ) -> Self {
        Self {
            metadata,
            package_id,
            dep_name,
        }
    }

    pub fn build(&self) -> Graph {
        let resolver = PackageResolver::new(self.metadata);
        let Some(dep_package) = resolver.resolve_package(self.package_id, self.dep_name) else {
            unimplemented!("Could not resolve package: {:?}", self.dep_name);
        };

        let mut node_indexes = HashMap::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for (feature, dep_features) in &dep_package.features {
            let from_index = node_indexes
                .entry(feature)
                .or_insert_with(|| {
                    let index = nodes.len();
                    let id = format!("node_{index}");
                    nodes.push(Node::new(id.clone(), feature.clone()));
                    id
                })
                .clone();

            for dep_feature in dep_features {
                let to_node_index = node_indexes.entry(dep_feature).or_insert_with(|| {
                    let index = nodes.len();
                    let id = format!("node_{index}");
                    nodes.push(Node::new(id.clone(), dep_feature.clone()));
                    id
                });

                edges.push(Edge::from_to(from_index.clone(), to_node_index.as_str()));
            }
        }

        Graph::new(nodes, edges)
    }
}