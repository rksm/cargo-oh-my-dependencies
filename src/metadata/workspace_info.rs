use eyre::Result;
use std::{path::PathBuf, rc::Rc};

use cargo_metadata::{Metadata, Package, PackageId};

use crate::cargo;

use super::{Features, PackageResolver};

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub manifest_path: PathBuf,
    pub metadata: Rc<Metadata>,
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

        Ok(Self {
            manifest_path,
            metadata,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        self.metadata = Rc::new(
            cargo_metadata::MetadataCommand::new()
                .manifest_path(&self.manifest_path)
                .exec()?,
        );
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

    pub fn package_resolver(&self) -> PackageResolver<'_> {
        PackageResolver::new(&self.metadata)
    }
}
