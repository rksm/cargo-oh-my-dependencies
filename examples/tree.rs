use eyre::Result;

use cargo_oh_my_dependencies::metadata::workspace_info::WorkspaceInfo;

fn main() -> Result<()> {
    color_eyre::install().expect("color_eyre init");

    let manifest_path = "/home/robert/temp/test-crate-workpspace/Cargo.toml";
    let info = WorkspaceInfo::load(manifest_path).expect("load workspace info");
    let tree = info.tree();

    tree.visit(&mut |node, parent| {
        println!("{:?}: {:?}", node, parent);
    });

    tree.visit_post_order(&mut |node, i, bottom| {
        if let Some(ids) = bottom {
            println!("[{i}] {:?}: {:?}", node, ids);
        } else {
            println!("[{i}] {:?}: None", node);
        }
        node.widget_id()
    });

    Ok(())
}
