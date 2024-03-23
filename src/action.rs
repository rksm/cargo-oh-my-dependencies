use cargo_metadata::{DependencyKind, PackageId};
use eyre::Result;

use crate::metadata::dep_tree::FeatureStatus;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Action {
    Render,
    Resize(u16, u16),
    Quit,
    Help,

    ShowFeatureTree {
        parent_package: PackageId,
        dep_name: String,
    },

    ToggleFeature {
        parent_package: PackageId,
        dep_name: String,
        dep_kind: DependencyKind,
        feature_name: String,
        feature_status: FeatureStatus,
    },
}

impl Action {
    pub fn none() -> Result<Option<Self>> {
        Ok(None)
    }

    pub fn render() -> Result<Option<Self>> {
        Ok(Some(Action::Render))
    }

    pub fn quit() -> Result<Option<Self>> {
        Ok(Some(Action::Quit))
    }
}
