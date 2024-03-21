use cargo_metadata::{Metadata, Node, Package, PackageId};
use std::collections::BTreeMap;

/// Given a dependent package and a package name which is a dependency of the
/// dependent package, resolves to the actual package.
pub struct PackageResolver<'a> {
    resolved: BTreeMap<PackageId, &'a Node>,
    packages: BTreeMap<PackageId, &'a Package>,
}

impl<'a> PackageResolver<'a> {
    pub fn new(metadata: &'a Metadata) -> Self {
        let resolved = metadata
            .resolve
            .as_ref()
            .unwrap() // TODO
            .nodes
            .iter()
            .map(|n| (n.id.clone(), n))
            .collect();
        let packages = metadata
            .packages
            .iter()
            .map(|p| (p.id.clone(), p))
            .collect();
        Self { resolved, packages }
    }

    pub fn resolve_package(&self, dependent: &PackageId, package_name: &str) -> Option<&Package> {
        let Some(resolver) = self.resolved.get(dependent) else {
            return None;
        };
        resolver
            .deps
            .iter()
            .find(|d| {
                d.name == package_name
                    || (package_name.contains('-') && d.name == package_name.replace('-', "_"))
                    || (package_name.contains('_') && d.name == package_name.replace('_', "-"))
            })
            .and_then(|dep| self.packages.get(&dep.pkg))
            .copied()
    }
}
