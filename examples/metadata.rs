#[macro_use]
extern crate tracing;

use eyre::Result;
use std::collections::{BTreeMap, HashSet};

use cargo_oh_my_dependencies::metadata::{Features, PackageResolver};

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
