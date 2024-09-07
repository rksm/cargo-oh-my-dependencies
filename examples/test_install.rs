#![allow(unused_variables, unused_imports)]

use cargo_metadata::DependencyKind;
use eyre::Result;
use toml_edit::DocumentMut;

use toml_edit::visit::*;
use toml_edit::visit_mut::*;

use cargo_oh_my_dependencies::{cargo::EditDependency, metadata::workspace_info::WorkspaceInfo};

fn main() -> Result<()> {
    color_eyre::install().expect("color_eyre init");
    tracing_subscriber::fmt::init();

    let manifest_path = "/home/robert/temp/test-crate-workpspace/Cargo.toml";
    let manifest_path = "/home/robert/projects/biz/podwriter/Cargo.toml";
    let info = WorkspaceInfo::load(manifest_path).expect("load workspace info");

    let p = info.workspace_packages()[1];

    let root = info.metadata.root_package().expect("root package");
    let inherited = Some(root.manifest_path.clone().into_std_path_buf());

    EditDependency::new(p, "serde", DependencyKind::Normal)
        .toggle_feature("derive")
        .set_workspace_dependency_at(inherited)
        .dry_run()
        .apply()?;

    Ok(())
}
