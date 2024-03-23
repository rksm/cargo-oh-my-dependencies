use cargo_metadata::{DependencyKind, PackageId};

use super::{workspace_info::WorkspaceInfo, Features};

#[derive(Default, Debug, Clone)]
pub struct DepTree {
    pub items: Vec<DepTreeNode>,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone)]
pub enum DepTreeNode {
    WorkspacePackage {
        id: PackageId,
        children: Vec<usize>,
    },

    UnresolvedDependency {
        name: String,
        kind: DependencyKind,
    },

    Dependency {
        name: String,
        kind: DependencyKind,
        children: Vec<usize>,
    },

    Feature {
        name: String,
        status: FeatureStatus,
        deps: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum FeatureStatus {
    Enabled,
    IndirectlyEnabled,
    Disabled,
}

pub type PostOrderCallback<'a, 'b, T> =
    dyn (FnMut(&'a DepTreeNode, usize, Option<Vec<T>>) -> T) + 'b;

impl DepTreeNode {
    fn package(id: PackageId) -> Self {
        DepTreeNode::WorkspacePackage {
            id,
            children: Vec::new(),
        }
    }

    fn resolved(name: impl ToString, kind: DependencyKind) -> Self {
        DepTreeNode::Dependency {
            name: name.to_string(),
            kind,
            children: Vec::new(),
        }
    }

    fn unresolved(name: impl ToString, kind: DependencyKind) -> Self {
        DepTreeNode::UnresolvedDependency {
            name: name.to_string(),
            kind,
        }
    }

    fn feature(name: impl ToString, status: FeatureStatus, deps: Vec<String>) -> Self {
        DepTreeNode::Feature {
            name: name.to_string(),
            status,
            deps,
        }
    }

    fn children(&self) -> Option<&Vec<usize>> {
        match self {
            DepTreeNode::WorkspacePackage { children, .. } => Some(children),
            DepTreeNode::Dependency { children, .. } => Some(children),
            _ => None,
        }
    }

    fn set_children(&mut self, children: Vec<usize>) {
        match self {
            DepTreeNode::WorkspacePackage {
                children: ref mut c,
                ..
            } => *c = children,
            DepTreeNode::Dependency {
                children: ref mut c,
                ..
            } => *c = children,
            _ => {}
        }
    }

    pub fn widget_id(&self) -> String {
        match self {
            DepTreeNode::WorkspacePackage { id, .. } => id.to_string(),
            DepTreeNode::UnresolvedDependency { name, kind, .. } => format!("{name}:{kind}"),
            DepTreeNode::Dependency { name, kind, .. } => format!("{name}:{kind}"),
            DepTreeNode::Feature { name, .. } => name.clone(),
        }
    }

    pub fn visit(
        &self,
        parent: Option<&DepTreeNode>,
        items: &[DepTreeNode],
        visitor: &mut dyn FnMut(&DepTreeNode, Option<&DepTreeNode>),
    ) {
        visitor(self, parent);

        if let Some(children) = self.children() {
            for i in children {
                let child = &items[*i];
                child.visit(Some(self), items, visitor);
            }
        }
    }

    fn visit_post_order<'a, 'b, 's, T>(
        &'s self,
        i: usize,
        items: &'a [DepTreeNode],
        visitor: &mut PostOrderCallback<'a, 'b, T>,
    ) -> T
    where
        's: 'a,
    {
        let input = self.children().map(|children| {
            children
                .iter()
                .map(|i| {
                    let child = &items[*i];
                    child.visit_post_order(*i, items, visitor)
                })
                .collect()
        });

        visitor(self, i, input)
    }
}

impl DepTree {
    pub fn build(workspace_info: &WorkspaceInfo) -> Self {
        let resolver = workspace_info.dependency_resolver();
        let mut items = Vec::new();
        let mut children = Vec::new();

        for p in &workspace_info.workspace_packages() {
            let i = items.len();
            children.push(items.len());

            let mut children = Vec::new();
            items.push(DepTreeNode::package(p.id.clone()));

            for dep in &p.dependencies {
                let i = items.len();
                children.push(i);
                let Some(dep_package) = resolver.resolve_dependency(&p.id, &dep.name) else {
                    warn!("Could not resolve package {}", dep.name);
                    items.push(DepTreeNode::unresolved(&dep.name, dep.kind));
                    continue;
                };
                items.push(DepTreeNode::resolved(dep.name.clone(), dep.kind));

                let mut children = Vec::new();
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

                    children.push(items.len());
                    items.push(DepTreeNode::feature(feature, status, feature_deps.clone()));
                }

                items.get_mut(i).unwrap().set_children(children);
            }

            items.get_mut(i).unwrap().set_children(children);
        }

        DepTree { items, children }
    }

    pub fn visit(&self, visitor: &mut dyn FnMut(&DepTreeNode, Option<&DepTreeNode>)) {
        for i in &self.children {
            let item = &self.items[*i];
            item.visit(None, &self.items, visitor);
        }
    }

    pub fn visit_post_order<'s, 'a, 'b, T>(
        &'s self,
        visitor: &mut PostOrderCallback<'a, 'b, T>,
    ) -> Vec<T>
    where
        's: 'a,
    {
        self.children
            .iter()
            .map(|i| {
                let item = &self.items[*i];
                item.visit_post_order(*i, &self.items, visitor)
            })
            .collect()
    }
}
