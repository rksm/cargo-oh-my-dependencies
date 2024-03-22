#[macro_use]
extern crate tracing;

use eyre::Result;
use std::collections::{BTreeMap, HashSet};

use cargo_oh_my_dependencies::metadata::{dep_tree, workspace_info::WorkspaceInfo};

fn main() -> Result<()> {
    color_eyre::install().expect("color_eyre init");

    let manifest_path = "/home/robert/temp/test-crate-workpspace/Cargo.toml";
    let info = WorkspaceInfo::load(manifest_path).expect("load workspace info");

    let tree = dep_tree::DepTree::build(&info);

    dbg!(tree);

    Ok(())
}
