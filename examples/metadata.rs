#[macro_use]
extern crate tracing;

use eyre::Result;
use std::collections::{BTreeMap, HashSet};

use cargo_metadata::{Dependency, Metadata, Node, Package, PackageId};

/// Given a dependent package and a package name which is a dependency of the
/// dependent package, resolves to the actual package.
struct PackageResolver<'a> {
    resolved: BTreeMap<PackageId, &'a Node>,
    packages: BTreeMap<PackageId, &'a Package>,
}

impl<'a> PackageResolver<'a> {
    fn new(metadata: &'a Metadata) -> Self {
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

    fn resolve_package(&self, dependent: &PackageId, package_name: &str) -> Option<&Package> {
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

/// Computes active and indirectly active features given a dependency
/// (metadata.package.dependencies) and a resolved package.
struct Features<'a> {
    dependency: &'a Dependency,
    package: &'a Package,
}

impl<'a> Features<'a> {
    fn new(dependency: &'a Dependency, package: &'a Package) -> Self {
        Self {
            dependency,
            package,
        }
    }

    fn active_features(&self) -> HashSet<&'a String> {
        self.package
            .features
            .iter()
            .filter_map(|(feature, _)| {
                if self.dependency.features.contains(feature)
                    || (feature == "default" && self.dependency.uses_default_features)
                {
                    Some(feature)
                } else {
                    None
                }
            })
            .collect()
    }

    fn indirectly_active_features(&self) -> HashSet<&'a String> {
        let mut indirectly_active_features = self.active_features();
        let mut last_feature_count = 0;
        while last_feature_count != indirectly_active_features.len() {
            last_feature_count = indirectly_active_features.len();
            for (feature, feature_deps) in &self.package.features {
                if indirectly_active_features.contains(feature) {
                    indirectly_active_features.extend(feature_deps);
                }
            }
        }
        indirectly_active_features
    }
}

fn main() -> Result<()> {
    color_eyre::install().expect("color_eyre init");

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()?;

    let workspace_package_ids = metadata.workspace_members.iter().collect::<HashSet<_>>();

    let packages = metadata
        .packages
        .iter()
        .map(|p| (p.id.clone(), p))
        .collect::<BTreeMap<_, _>>();

    let resolver = PackageResolver::new(&metadata);

    for id in workspace_package_ids {
        let p = packages.get(id).unwrap(); // TODO

        println!("{}", p.name);
        for dep in &p.dependencies {
            let Some(dep_package) = resolver.resolve_package(id, &dep.name) else {
                warn!("Could not resolve package {}", dep.name);
                println!("! {}", dep.name);
                continue;
            };

            let features = Features::new(dep, dep_package);
            let active_features = features.active_features();
            let indirectly_active_features = features.indirectly_active_features();

            println!("  {}", dep.name);
            for (feature, feature_deps) in &dep_package.features {
                let marker = if active_features.contains(feature) {
                    "x"
                } else if indirectly_active_features.contains(feature) {
                    "!"
                } else {
                    " "
                };

                print!("    [{marker}] {feature}");
                if !feature_deps.is_empty() {
                    print!(" ({})", feature_deps.join(", "));
                }
                println!();
            }
        }
    }

    Ok(())
}
