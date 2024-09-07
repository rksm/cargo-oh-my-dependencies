use std::collections::BTreeSet;

use cargo_oh_my_dependencies::metadata::toml::DebugVisitor;
use eyre::Result;
use toml_edit::DocumentMut;

use toml_edit::visit::*;
use toml_edit::visit_mut::*;

use cargo_oh_my_dependencies::metadata::toml;
use cargo_oh_my_dependencies::metadata::workspace_info::WorkspaceInfo;

fn main() -> Result<()> {
    color_eyre::install().expect("color_eyre init");
    tracing_subscriber::fmt::init();

    let manifest_path = "/home/robert/temp/test-crate/Cargo.toml";

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

    if false {
        let info = WorkspaceInfo::load(manifest_path)?;
        // dbg!(info.manifest);
        dbg!(info.metadata);
    }
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

    if false {
        let mut doc: DocumentMut = std::fs::read_to_string(manifest_path)?.parse()?;

        // dbg!(visit_example(&doc));
        visit_example2(&mut doc);

        println!("{doc}");
    }
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

    if true {
        let doc: DocumentMut = std::fs::read_to_string(manifest_path)?.parse()?;
        DebugVisitor::default().visit_document(&doc);
    }

    Ok(())
}

#[allow(dead_code)]
fn visit_example(document: &DocumentMut) -> BTreeSet<&str> {
    let mut visitor = toml::DependencyNameVisitor {
        state: toml::VisitState::Root,
        names: BTreeSet::new(),
    };

    visitor.visit_document(document);

    visitor.names
}

fn visit_example2(document: &mut DocumentMut) {
    let mut visitor = toml::FeatureAddVisitor {
        state: toml::VisitState::Root,
        dep: Default::default(),
        feature: Default::default(),
        kind: Default::default(),
    };

    visitor.visit_document_mut(document);
}
