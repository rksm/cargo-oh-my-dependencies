use cargo_metadata::{Dependency, Package};
use std::collections::HashSet;

/// Computes active and indirectly active features given a dependency
/// (metadata.package.dependencies) and a resolved package.
pub struct Features<'a> {
    dependency: &'a Dependency,
    package: &'a Package,
}

impl<'a> Features<'a> {
    pub fn new(dependency: &'a Dependency, package: &'a Package) -> Self {
        Self {
            dependency,
            package,
        }
    }

    pub fn active_features(&self) -> HashSet<&'a String> {
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

    pub fn indirectly_active_features(&self) -> HashSet<&'a String> {
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
