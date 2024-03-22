use cargo_metadata::PackageId;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
    Render,
    Resize(u16, u16),
    Quit,
    Error(String),
    Help,

    ShowFeatureTree {
        parent_package: PackageId,
        dep_name: String,
    },
}
