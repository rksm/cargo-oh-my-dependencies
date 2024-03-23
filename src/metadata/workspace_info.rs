use eyre::Result;
use std::{path::PathBuf, rc::Rc};

use cargo_metadata::{Metadata, Package, PackageId};
use cargo_toml::Manifest;

use crate::cargo;

use super::{dep_tree::DepTree, PackageResolver};

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub manifest_path: PathBuf,
    metadata: Rc<Metadata>,
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
        pkg: &PackageId,
        dep_name: &str,
        feature_name: &str,
    ) -> Result<()> {
        let Some(package) = self.metadata.packages.iter().find(|p| &p.id == pkg) else {
            eyre::bail!("Package not found");
        };

        cargo::EditDependency::new(package, dep_name)
            .toggle_feature(feature_name)
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
