use std::collections::{BTreeMap, HashSet};

use eyre::Result;

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub features: BTreeMap<String, Vec<String>>,
}

fn main() -> Result<()> {
    color_eyre::install().expect("color_eyre init");

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()?;

    let workspace_package_ids = metadata
        .workspace_members
        .into_iter()
        .collect::<HashSet<_>>();
    let mut packages = BTreeMap::new();

    let resolved = metadata
        .resolve
        .unwrap()
        .nodes
        .into_iter()
        .map(|n| (n.id.clone(), n))
        .collect::<BTreeMap<_, _>>();

    for p in metadata.packages {
        // let package = Package {
        //     name: p.name,
        //     version: p.version.to_string(),
        //     features: p.features,
        // };
        packages.insert(p.id.clone(), p);
    }

    for id in workspace_package_ids {
        let p = &packages.get(&id).unwrap();
        let resolver = resolved.get(&id).unwrap();
        let resolved_deps = resolver
            .deps
            .iter()
            .map(|d| (&d.name, &d.pkg))
            .collect::<BTreeMap<_, _>>();

        println!("{}", p.name);
        for dep in &p.dependencies {
            let Some(dep_package) = resolved_deps
                .get(&dep.name)
                .or_else(|| {
                    if dep.name.contains('-') {
                        resolved_deps.get(&dep.name.replace('-', "_"))
                    } else if dep.name.contains('_') {
                        resolved_deps.get(&dep.name.replace('_', "-"))
                    } else {
                        None
                    }
                })
                .and_then(|id| packages.get(id))
            else {
                println!("! {}", dep.name);
                continue;
            };

            let active_features = dep_package
                .features
                .iter()
                .filter_map(|(feature, _)| {
                    if dep.features.contains(feature)
                        || (feature == "default" && dep.uses_default_features)
                    {
                        Some(feature)
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>();

            let mut indirectly_active_features = active_features.clone();
            let mut last_feature_count = 0;
            while last_feature_count != indirectly_active_features.len() {
                last_feature_count = indirectly_active_features.len();
                for (feature, feature_deps) in &dep_package.features {
                    if indirectly_active_features.contains(feature) {
                        indirectly_active_features.extend(feature_deps);
                    }
                }
            }

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
