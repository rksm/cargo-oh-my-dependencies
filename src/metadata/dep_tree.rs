use std::collections::HashMap;

use cargo_metadata::{Dependency, DependencyKind, PackageId};

use super::{workspace_info::WorkspaceInfo, Features};

#[derive(Default, Debug, Clone)]
pub struct DepTree {
    pub index: HashMap<String, usize>,
    pub items: Vec<DepTreeNode>,
}

#[derive(Debug, Clone)]
enum DepTreeNode {
    WorkspacePackage {
        id: PackageId,
    },

    UnresolvedDependency {
        id: String,
        package_id: PackageId,
        name: String,
        kind: DependencyKind,
    },

    Dependency {
        id: String,
        package_id: PackageId,
        name: String,
        kind: DependencyKind,
    },

    Feature {
        id: String,
        dependency_id: String,
        name: String,
        status: FeatureStatus,
        dep_features: Vec<String>,
    },
}

#[derive(Debug, Clone)]
enum FeatureStatus {
    Enabled,
    IndirectlyEnabled,
    Disabled,
}

impl DepTreeNode {
    pub fn resolved(
        id: impl ToString,
        package_id: PackageId,
        name: impl ToString,
        kind: DependencyKind,
    ) -> Self {
        DepTreeNode::Dependency {
            id: id.to_string(),
            package_id,
            name: name.to_string(),
            kind,
        }
    }

    pub fn unresolved(
        id: impl ToString,
        package_id: PackageId,
        name: impl ToString,
        kind: DependencyKind,
    ) -> Self {
        DepTreeNode::UnresolvedDependency {
            id: id.to_string(),
            package_id,
            name: name.to_string(),
            kind,
        }
    }
}

impl DepTree {
    pub fn build(workspace_info: &WorkspaceInfo) -> Self {
        let resolver = workspace_info.package_resolver();
        let mut items = Vec::new();
        let mut index = HashMap::new();

        for p in &workspace_info.workspace_packages() {
            let key = p.id.to_string();
            index.insert(key.clone(), items.len());
            items.push(DepTreeNode::WorkspacePackage { id: p.id.clone() });

            for dep in &p.dependencies {
                let tree_key = format!("{}:{}:{}", p.id, dep.name, dep.kind);

                index.insert(tree_key.clone(), items.len());
                let Some(dep_package) = resolver.resolve_package(&p.id, &dep.name) else {
                    warn!("Could not resolve package {}", dep.name);
                    items.push(DepTreeNode::unresolved(
                        tree_key.clone(),
                        p.id.clone(),
                        &dep.name,
                        dep.kind,
                    ));
                    continue;
                };

                items.push(DepTreeNode::resolved(
                    tree_key.clone(),
                    p.id.clone(),
                    dep.name.clone(),
                    dep.kind,
                ));

                let features = Features::new(dep, dep_package);
                let active_features = features.active_features();
                let indirectly_active_features = features.indirectly_active_features();

                for (feature, feature_deps) in dep_package.features.iter() {
                    let status = if active_features.contains(feature) {
                        FeatureStatus::Enabled
                    } else if indirectly_active_features.contains(feature) {
                        FeatureStatus::IndirectlyEnabled
                    } else {
                        FeatureStatus::Disabled
                    };

                    let key = format!("{tree_key}:{feature}");
                    index.insert(key.clone(), items.len());
                    items.push(DepTreeNode::Feature {
                        id: key,
                        dependency_id: tree_key.clone(),
                        name: feature.clone(),
                        status,
                        dep_features: feature_deps.clone(),
                    });
                }
            }
        }

        DepTree { items, index }
    }

    pub fn visit(&self, visitor: &mut dyn FnMut(&DepTreeNode)) {
        for item in &self.items {
            visitor(item);
        }
    }
}
