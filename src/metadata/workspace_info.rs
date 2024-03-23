use eyre::Result;
use std::{path::PathBuf, rc::Rc};

use cargo_metadata::{DependencyKind, Metadata, Package, PackageId};
use cargo_toml::Manifest;

use crate::cargo;

use super::{
    dep_tree::{DepTree, FeatureStatus},
    PackageResolver,
};

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub manifest_path: PathBuf,
    pub metadata: Rc<Metadata>,
    pub manifest: Manifest,
}

impl WorkspaceInfo {
    pub fn load(manifest_path: impl Into<PathBuf>) -> Result<Self> {
        let mut manifest_path = manifest_path.into();
        if manifest_path.is_dir() {
            manifest_path.push("Cargo.toml");
        }
        if !manifest_path.exists() {
            eyre::bail!("{manifest_path:?} not found");
        }

        let metadata = Rc::new(
            cargo_metadata::MetadataCommand::new()
                .manifest_path(&manifest_path)
                .exec()?,
        );

        let manifest = Manifest::from_path(&manifest_path)?;

        Ok(Self {
            manifest_path,
            metadata,
            manifest,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        self.metadata = Rc::new(
            cargo_metadata::MetadataCommand::new()
                .manifest_path(&self.manifest_path)
                .exec()?,
        );
        self.manifest = Manifest::from_path(&self.manifest_path)?;
        Ok(())
    }

    pub fn toggle_feature(
        &mut self,
        pkg: PackageId,
        dep_name: String,
        dep_kind: DependencyKind,
        feature_name: String,
        _feature_status: FeatureStatus,
    ) -> Result<()> {
        let Some(package) = self.metadata.packages.iter().find(|p| p.id == pkg) else {
            eyre::bail!("Package not found");
        };

        let manifest_deps = match dep_kind {
            DependencyKind::Normal | DependencyKind::Unknown => &self.manifest.dependencies,
            DependencyKind::Development => &self.manifest.dev_dependencies,
            DependencyKind::Build => &self.manifest.build_dependencies,
        };
        let dep = manifest_deps.get(&dep_name).ok_or_else(|| {
            eyre::eyre!(
                "Dependency {dep_name:?} not found in manifest {:?}",
                self.manifest_path
            )
        })?;

        let inherited = dep.detail().and_then(|d| {
            if d.inherited {
                Some(&self.metadata.workspace_root)
            } else {
                None
            }
        });

        cargo::EditDependency::new(package, &dep_name, dep_kind)
            .toggle_feature(feature_name)
            .set_workspace_dependency_at(inherited)
            .apply()?;

        Ok(())
    }

    pub fn workspace_packages(&self) -> Vec<&Package> {
        self.metadata.workspace_packages()
    }

    pub fn dependency_resolver(&self) -> PackageResolver<'_> {
        PackageResolver::new(&self.metadata)
    }

    pub fn tree(&self) -> DepTree {
        DepTree::build(self)
    }
}
